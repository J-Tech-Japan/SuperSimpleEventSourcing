use std::any::Any;
use std::fmt;
use std::fmt::Debug;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

pub trait EventPayload: Any + Debug {
    fn as_any(&self) -> &dyn Any;
    fn clone_box(&self) -> Box<dyn EventPayload>;
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

// 30 charactors string that contains 19 charactors of ticks and 11 charactors of id
pub struct SortableUniqueIdValue(pub String);

impl SortableUniqueIdValue {
    pub const SAFE_MILLISECONDS: i64 = 5000;
    pub const TICK_NUMBER_OF_LENGTH: usize = 19;
    pub const ID_NUMBER_OF_LENGTH: usize = 11;

    pub fn new(value: &str) -> Self {
        Self(value.to_string())
    }

    pub fn get_ticks(&self) -> SystemTime {
        let ticks_str = &self.0[..Self::TICK_NUMBER_OF_LENGTH];
        let ticks = ticks_str.parse::<u64>().unwrap();
        UNIX_EPOCH + Duration::from_nanos(ticks * 100)
    }

    pub fn generate(timestamp: SystemTime, id: Uuid) -> Self {
        Self(Self::get_tick_string(timestamp) + &Self::get_id_string(id))
    }

    pub fn get_safe_id_from_utc() -> Self {
        let now = SystemTime::now()
            .checked_sub(Duration::from_millis(Self::SAFE_MILLISECONDS as u64))
            .unwrap();
        Self(Self::get_tick_string(now) + &Self::get_id_string(Uuid::nil()))
    }

    pub fn get_current_id_from_utc() -> Self {
        let now = SystemTime::now();
        Self(Self::get_tick_string(now) + &Self::get_id_string(Uuid::nil()))
    }

    pub fn get_safe_id(&self) -> Self {
        let safe_time = self
            .get_ticks()
            .checked_sub(Duration::from_millis(Self::SAFE_MILLISECONDS as u64))
            .unwrap();
        Self(Self::get_tick_string(safe_time) + &Self::get_id_string(Uuid::nil()))
    }

    pub fn is_earlier_than(&self, to_compare: &Self) -> bool {
        self.0 < to_compare.0
    }

    pub fn is_earlier_than_or_equal(&self, to_compare: &Self) -> bool {
        self.0 <= to_compare.0
    }

    pub fn is_later_than(&self, to_compare: &Self) -> bool {
        self.0 > to_compare.0
    }

    pub fn is_later_than_or_equal(&self, to_compare: &Self) -> bool {
        self.0 >= to_compare.0
    }

    fn get_tick_string(timestamp: SystemTime) -> String {
        let ticks = SortableUniqueIdValue::system_time_to_csharp_ticks(timestamp);
        format!("{:019}", ticks)
    }
    const TICKS_PER_SECOND: u64 = 10_000_000; // 1秒あたりのC#ティック数 (100ナノ秒単位)
    const TICKS_FROM_UNIX_TO_CSHARP: u64 = 621_355_968_000_000_000; // UNIX_EPOCHから1/1/0001までのティック数

    pub fn system_time_to_csharp_ticks(timestamp: SystemTime) -> u64 {
        let duration_since_unix = timestamp
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::ZERO);

        let ticks_since_unix = duration_since_unix.as_secs() * SortableUniqueIdValue::TICKS_PER_SECOND
            + (duration_since_unix.subsec_nanos() as u64 / 100); // 100ナノ秒単位に変換

        ticks_since_unix + SortableUniqueIdValue::TICKS_FROM_UNIX_TO_CSHARP
    }
    fn get_id_string(id: Uuid) -> String {
        let id_hash = id.to_u128_le() as i64; // Convert UUID to a hash
        let id_mod_base = 10_i64.pow(Self::ID_NUMBER_OF_LENGTH as u32);
        format!("{:011}", id_hash.abs() % id_mod_base)
    }
}

impl From<&str> for SortableUniqueIdValue {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<SortableUniqueIdValue> for String {
    fn from(suid: SortableUniqueIdValue) -> Self {
        suid.0
    }
}

pub trait Command { }

pub trait CommandWithHandler : Command {
    fn get_projector(&self) -> Box<dyn AggregateProjector>;
    fn get_partition_keys(&self) -> PartitionKeys;
    fn command_handler(&self, context: &CommandContext) -> Option<Box<dyn EventPayload>>;
}

#[derive(Debug)]
pub struct CommandResponse {
    pub partition_keys: PartitionKeys,
    pub events: Vec<EventCommon>,
    pub version: i64,
}

pub trait CommandContextTrait {
    fn get_events(&self) -> &[EventCommon];
    fn get_current_aggregate(&self) -> Aggregate;
    fn save_event<TEventPayload: EventPayload + Clone>(&mut self, event_payload: TEventPayload) -> Option<Box<dyn EventPayload>>;
}

pub struct CommandContext<'a> {
    pub events:  &'a mut Vec<EventCommon>,
    pub aggregate: &'a mut Aggregate,
    pub projector: &'a dyn AggregateProjector,
}
impl CommandContextTrait for CommandContext<'_> {
    fn get_events(&self) -> &[EventCommon] {
        &self.events
    }

    fn get_current_aggregate(&self) -> Aggregate {
        self.aggregate.clone()
    }

    fn save_event<TEventPayload: EventPayload + Clone>(&mut self, event_payload:TEventPayload) -> Option<Box<dyn EventPayload>>
    {
        let event = Box::new(EventCommon {
            payload: Box::new(event_payload),
            partition_keys: self.aggregate.partition_keys.clone(),
            sortable_unique_id: SortableUniqueIdValue::generate(SystemTime::now(), Uuid::new_v4()).into(),
            version: self.aggregate.version + 1,
        });
        *self.aggregate = self.aggregate.project(&*event, &*self.projector);
        self.events.push(*event);
        None
    }
}


pub struct CommandExecutor {
    pub repository: Repository,
}

impl CommandExecutor {
    pub fn execute<TCommand: Command>(
        &mut self, command: TCommand,
        projector: &dyn AggregateProjector,
        partition_keys_provider: fn(&TCommand) -> PartitionKeys,
        command_handler: fn(&TCommand, &CommandContext) -> Option<Box<dyn EventPayload>>) -> CommandResponse
        {
        let partition_keys = partition_keys_provider(&command);
        let mut current_aggregate = self.repository.load(&partition_keys, projector)
            .unwrap_or_else(|_| Aggregate::empty_from_partition_keys(partition_keys.clone()));
            let mut events : Vec<EventCommon> = Vec::new();
        let context = CommandContext {
            events: &mut events,
            aggregate: &mut current_aggregate,
            projector: projector,
        };
            // if event is some, push event to context.events, if not, get context.events
            if let Some(event_payload) = command_handler(&command, &context) {
                let event_payload2 = event_payload.clone_box();
                let last_event = EventCommon {
                    payload: event_payload2,
                    partition_keys: context.aggregate.partition_keys.clone(),
                    sortable_unique_id: SortableUniqueIdValue::generate(SystemTime::now(), Uuid::new_v4()).into(),
                    version: context.aggregate.version + 1,
                };
                context.events.push(last_event);
            }
            let saved_events: Vec<EventCommon> = context.events
                .iter()
                .map(|event| event.clone_event_common())
                .collect();
            let _ = self.repository.save_events(saved_events).unwrap();
            CommandResponse {
                partition_keys: context.aggregate.partition_keys.clone(),
                events: context.events.iter().map(|event| event.clone_event_common()).collect(),
                version: context.aggregate.version,
            }
    }
}
#[derive(Debug)]
pub struct EventCommon {
    pub version: i64,
    pub sortable_unique_id: String,
    pub partition_keys: PartitionKeys,
    pub payload: Box<dyn EventPayload>,
}
impl EventCommon {
    pub fn clone_event_common(&self) -> EventCommon {
        EventCommon {
            version: self.version,
            sortable_unique_id: self.sortable_unique_id.clone(),
            partition_keys: self.partition_keys.clone(),
            payload: self.payload.clone_box(),
        }
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

    pub fn project(&self, ev: &EventCommon, projector: &dyn AggregateProjector) -> Self {
        Self {
            payload: projector.project(&*self.payload, ev),
            last_sortable_unique_id: ev.sortable_unique_id.clone(),
            version: ev.version,
            ..self.clone() // Ensure `Clone` is implemented for `Aggregate`
        }
    }

    pub fn project_all(&self, events: &[EventCommon], projector: &dyn AggregateProjector) -> Self {
        events.iter().fold(self.clone(), |acc, ev| acc.project(ev, projector))
    }
}
impl Clone for Aggregate {
    fn clone(&self) -> Self {
        Self {
            payload: self.payload.clone_box(), // Use the `clone_box` method
            partition_keys: self.partition_keys.clone(),
            version: self.version,
            last_sortable_unique_id: self.last_sortable_unique_id.clone(),
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
    fn project(&self, payload: &dyn AggregatePayload, ev: &EventCommon) -> Box<dyn AggregatePayload>;

    fn get_version(&self) -> &str {
        "initial"
    }
    fn clone_box(&self) -> Box<dyn AggregateProjector>;
}


pub struct Repository {
    events: Vec<EventCommon>,
}

impl Repository {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
        }
    }
    pub fn load(&self, partition_keys: &PartitionKeys, projector: &dyn AggregateProjector) -> Result<Aggregate, String> {
        let mut filtered: Vec<&EventCommon> = self.events
            .iter()
            .filter(|ev| {
                ev.partition_keys.aggregate_id == partition_keys.aggregate_id
                    && ev.partition_keys.group_ == partition_keys.group_
                    && ev.partition_keys.root_partition_key == partition_keys.root_partition_key
            })
            .collect();
        filtered.sort_by_key(|ev| ev.sortable_unique_id.to_string());
        let aggregate = Aggregate::empty_from_partition_keys(partition_keys.clone());
        let projected = filtered.iter().fold(aggregate, |acc, ev| acc.project(ev, projector));
        Ok(projected)
    }

    pub fn save(&mut self, new_event: EventCommon) -> Result<(), String> {
        self.events.push(new_event);
        Ok(())
    }
    pub fn save_events(&mut self, mut new_events: Vec<EventCommon>) -> Result<(), String> {
        // iterate new events and call save method
        for event in new_events.iter_mut() {
            let _ = self.save(event.clone_event_common());
        }
        Ok(())
    }
}









#[derive(Debug,Clone)]
pub struct BranchCreated {
    pub name: String,
    pub country: String,
}
impl EventPayload for BranchCreated {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn EventPayload> {
        Box::new(self.clone())
    }
}
#[derive(Debug, Clone)]
pub struct BranchNameChanged {
    pub name: String,
}

impl EventPayload for BranchNameChanged {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn clone_box(&self) -> Box<dyn EventPayload> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct BranchCountryNameChanged {
    pub country: String,
}
impl EventPayload for BranchCountryNameChanged {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn EventPayload> {
        Box::new(self.clone())
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
#[derive(Clone)]
pub struct BranchProjector {}


impl AggregateProjector for BranchProjector {
    // if event.get_payload() is BranchCreated, then return Branch with name and country set
    // if event.get_payload() is BranchNameChanged, then return Branch with name set
    // if else, just return payload that passed as argument
    fn project(&self, payload: &dyn AggregatePayload, ev: &EventCommon) -> Box<dyn AggregatePayload> {
        // EmptyAggregatePayloadの場合の処理
        if let Some(_) = payload.as_any().downcast_ref::<EmptyAggregatePayload>() {
            let event = ev.payload.clone_box();
            if let Some(branch_created) = event.as_any().downcast_ref::<BranchCreated>() {
                return Box::new(Branch {
                    name: branch_created.name.clone(),
                    country: branch_created.country.clone(),
                });
            } else {
                // EmptyAggregatePayload だが BranchCreated でなかった場合は変化なし
                return (*payload).clone_box();
            }
        }

        // 既存のBranchがある場合の処理
        if let Some(branch) = payload.as_any().downcast_ref::<Branch>() {
            let event = ev.payload.clone_box();
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
            } else if let Some(branch_country_name_changed) = event.as_any().downcast_ref::<BranchCountryNameChanged>() {
                Box::new(Branch {
                    name: branch.name.clone(),
                    country: branch_country_name_changed.country.clone(),
                })
            } else {
                Box::new(Branch {
                    name: branch.name.clone(),
                    country: branch.country.clone(),
                })
            }
        } else {
            // Branch でも EmptyAggregatePayload でもない場合はそのまま
            (*payload).clone_box()
        }
    }

    fn clone_box(&self) -> Box<dyn AggregateProjector> {
        Box::new(self.clone())
    }
}

pub struct CreateBranchCommand {
    pub name: String,
    pub country: String,
}
impl Command for CreateBranchCommand {}

pub struct ChangeBranchNameCommand {
    pub name: String,
    pub partition_keys: PartitionKeys
}
impl Command for ChangeBranchNameCommand {}
#[derive(Clone)]
pub struct ChangeBranchCountryNameCommand {
    pub country: String,
    pub partition_keys: PartitionKeys
}

impl Command for ChangeBranchCountryNameCommand {}
impl CommandWithHandler for ChangeBranchCountryNameCommand {
    fn get_projector(&self) -> Box<dyn AggregateProjector> {
        Box::new(BranchProjector {})
    }

    fn get_partition_keys(&self) -> PartitionKeys {
        self.partition_keys.clone()
    }

    fn command_handler(&self, context: &CommandContext) -> Option<Box<dyn EventPayload>> {
        let binding = context.get_current_aggregate();
        let branch = binding.payload.as_any().downcast_ref::<Branch>().unwrap();
        if branch.country == self.country {
            return None;
        }
        Some(Box::new(BranchCountryNameChanged {
            country: self.country.clone(),
        }))
    }
}