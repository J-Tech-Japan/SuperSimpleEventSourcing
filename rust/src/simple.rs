use std::any::Any;
use std::fmt;
use uuid::Uuid;

pub trait EventPayload: Any {
    fn as_any(&self) -> &dyn Any;
}
pub trait AggregatePayload: fmt::Debug + Any {
    fn as_any(&self) -> &dyn Any;
    fn clone_box(&self) -> Box<dyn AggregatePayload>;
}
#[derive(Debug, Clone)]
pub struct PartitionKeys {
    pub aggregate_id: Uuid,
    pub group_: String,
    pub root_partition_key: String,
}
impl PartitionKeys {
    pub fn from_aggregate_id(aggregate_id: Uuid) -> Self {
        Self {
            aggregate_id,
            group_: "default".to_string(),
            root_partition_key: "default".to_string(),
        }
    }
}

// SortableUniqueIdValue相当
pub struct SortableUniqueIdValue(String);

impl SortableUniqueIdValue {
    fn new(value: &str) -> Self {
        Self(value.to_string())
    }
}

pub trait EventCommon {
    fn version(&self) -> i64;
    fn sortable_unique_id(&self) -> &str;
    fn partition_keys(&self) -> &PartitionKeys;
    fn get_sortable_unique_id(&self) -> SortableUniqueIdValue {
        // C# の default interface method に相当するデフォルト実装
        SortableUniqueIdValue::new(self.sortable_unique_id())
    }
    fn get_payload(&self) -> &dyn EventPayload;
}

#[derive(Debug)]
pub struct Event<TEventPayload: EventPayload> {
    pub payload: TEventPayload,
    pub partition_keys: PartitionKeys,
    pub sortable_unique_id: String,
    pub version: i64,
}

// IEventトレイト実装
impl<TEventPayload: EventPayload> EventCommon for Event<TEventPayload> {
    fn version(&self) -> i64 {
        self.version
    }

    fn sortable_unique_id(&self) -> &str {
        &self.sortable_unique_id
    }

    fn partition_keys(&self) -> &PartitionKeys {
        &self.partition_keys
    }

    fn get_payload(&self) -> &dyn EventPayload {
        &self.payload
    }
}

#[derive(Debug)]
pub struct Aggregate {
    pub payload: Box<dyn AggregatePayload>,
    pub partition_keys: PartitionKeys,
    pub version: i64,
    pub last_sortable_unique_id: String,
}
impl Aggregate {
    pub fn empty_from_partition_keys(partition_keys: PartitionKeys) -> Self {
        Self {
            payload: Box::new(EmptyAggregatePayload {}),
            partition_keys,
            version: 0,
            last_sortable_unique_id: "".to_string()
        }
    }
}
#[derive(Debug, Clone)]
pub struct EmptyAggregatePayload {}

impl AggregatePayload for EmptyAggregatePayload {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn AggregatePayload> {
        Box::new(self.clone())
    }
}

pub trait AggregateProjector {
    fn project(&self, payload: &dyn AggregatePayload, ev: &dyn EventCommon) -> Box<dyn AggregatePayload>;

    fn get_version(&self) -> &str {
        "initial"
    }
}



#[derive(Debug)]
pub struct BranchCreated {
    pub name: String,
    pub country: String,
}
impl EventPayload for BranchCreated {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
#[derive(Debug)]
pub struct BranchNameChanged {
    pub name: String,
}

impl EventPayload for BranchNameChanged {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Clone)]
pub struct Branch{
    pub name: String,
    pub country: String,
}

impl AggregatePayload for Branch {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn AggregatePayload> {
        Box::new(self.clone())
    }
}

pub struct BranchProjector {}


impl AggregateProjector for BranchProjector {
    // if event.get_payload() is BranchCreated, then return Branch with name and country set
    // if event.get_payload() is BranchNameChanged, then return Branch with name set
    // if else, just return payload that passed as argument
    fn project(&self, payload: &dyn AggregatePayload, ev: &dyn EventCommon) -> Box<dyn AggregatePayload> {
        if let Some(branch) = payload.as_any().downcast_ref::<Branch>() {
            let event = ev.get_payload();
            if let Some(branch_created) = event.as_any().downcast_ref::<BranchCreated>() {
                Box::new(Branch {
                    name: branch_created.name.clone(),
                    country: branch_created.country.clone(),
                })
            } else if let Some(branch_name_changed) = event.as_any().downcast_ref::<BranchNameChanged>() {
                Box::new(Branch {
                    name: branch_name_changed.name.clone(),
                    country: branch.country.clone(),
                })
            } else {
                Box::new(Branch {
                    name: branch.name.clone(),
                    country: branch.country.clone(),
                })
            }
        } else {
            (*payload).clone_box()
        }
    }
}