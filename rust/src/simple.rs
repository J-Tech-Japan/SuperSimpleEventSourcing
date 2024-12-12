use std::any::Any;
use std::fmt;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
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

// Fromトレイトを実装して文字列との暗黙的な変換を可能にする
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

pub trait ExecuteCommand<TCommand: Command> {
    fn execute(&self, command: TCommand) -> Result<(), String>;
}

pub struct CommandResponse {
    partition_keys: PartitionKeys,
    events: Vec<Box<dyn EventCommon>>,
    version: i64,
}

pub struct CommandContext {
    // EventCommonの配列
    pub events: Vec<Box<dyn EventCommon>>,
    // 現在のAggregateを取得する関数、入力はない、一部イベントを保存した場合は、保存後のAggregateを返す
    pub get_current_aggregate: fn() -> Aggregate,
    // Optional<EventCommon> を返し、入力は保存予定のEventCommon
    pub save_event: fn(Box<dyn EventCommon>) -> Option<Box<dyn EventCommon>>
}

pub struct CommandExecutor {

}

// impl CommandExecutor {
//     pub fn execute<TCommand: Command>(&self, command: TCommand, projector: &dyn AggregateProjector,
//                                       // TCommand入力でPartitionKeysを返す関数
//                                         partition_keys_provider: fn(TCommand) -> PartitionKeys,
//                                       // command handler コマンドとCommandContextとを受け取り、Optional<EventCommon>を返す関数
//                                         command_handler: fn(TCommand, CommandContext) -> Option<Box<dyn EventCommon>>) -> CommandResponse
//         {
//         let partition_keys = partition_keys_provider(command);
//         let current_aggregate = (partition_keys.aggregate_id, projector.get_version());
//         let context = CommandContext {
//             events: vec![],
//             get_current_aggregate: || current_aggregate,
//             save_event: |ev| Some(ev),
//         }
//     }
// }




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

    pub fn project(&self, ev: &dyn EventCommon, projector: &dyn AggregateProjector) -> Self {
        Self {
            payload: projector.project(&*self.payload, ev),
            last_sortable_unique_id: ev.sortable_unique_id().to_string(),
            version: ev.version(),
            ..self.clone() // Ensure `Clone` is implemented for `Aggregate`
        }
    }

    pub fn project_all(&self, events: &[Box<dyn EventCommon>], projector: &dyn AggregateProjector) -> Self {
        events.iter().fold(self.clone(), |acc, ev| acc.project(&**ev, projector))
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
    fn project(&self, payload: &dyn AggregatePayload, ev: &dyn EventCommon) -> Box<dyn AggregatePayload>;

    fn get_version(&self) -> &str {
        "initial"
    }
}


pub struct Repository {
    events: Vec<Box<dyn EventCommon>>,
}

impl Repository {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
        }
    }
    // C#でのLoad(PartitionKeys, IAggregateProjector)に対応
    pub fn load(&self, partition_keys: &PartitionKeys, projector: impl AggregateProjector) -> Result<Aggregate, String> {
        let mut filtered: Vec<&Box<dyn EventCommon>> = self.events
            .iter()
            .filter(|ev| {
                ev.partition_keys().aggregate_id == partition_keys.aggregate_id
                    && ev.partition_keys().group_ == partition_keys.group_
                    && ev.partition_keys().root_partition_key == partition_keys.root_partition_key
            })
            .collect();

        // SortableUniqueIdでソート
        filtered.sort_by_key(|ev| ev.sortable_unique_id().to_string());

        // 空のAggregateを開始点に、全イベントをProject
        let aggregate = Aggregate::empty_from_partition_keys(partition_keys.clone());
        let projected = filtered.iter().fold(aggregate, |acc, ev| acc.project(&***ev, &projector));

        Ok(projected)
    }

    // C#でのSave(List<IEvent>)に対応
    // EventsをRepositoryインスタンスに属するメンバとして拡張
    pub fn save(&mut self, new_events: Vec<Box<dyn EventCommon>>) -> Result<(), String> {
        self.events.extend(new_events);
        Ok(())
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
#[derive(Clone)]
pub struct BranchProjector {}


impl AggregateProjector for BranchProjector {
    // if event.get_payload() is BranchCreated, then return Branch with name and country set
    // if event.get_payload() is BranchNameChanged, then return Branch with name set
    // if else, just return payload that passed as argument
    fn project(&self, payload: &dyn AggregatePayload, ev: &dyn EventCommon) -> Box<dyn AggregatePayload> {
        // EmptyAggregatePayloadの場合の処理
        if let Some(_) = payload.as_any().downcast_ref::<EmptyAggregatePayload>() {
            let event = ev.get_payload();
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
            // Branch でも EmptyAggregatePayload でもない場合はそのまま
            (*payload).clone_box()
        }
    }
}