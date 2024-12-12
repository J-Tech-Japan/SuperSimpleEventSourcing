mod simple;

use std::time::SystemTime;
use uuid::Uuid;
use simple::BranchCreated;
use crate::simple::{Aggregate, AggregateProjector, Branch, BranchNameChanged, BranchProjector, Event, PartitionKeys, Repository, SortableUniqueIdValue};
use num_format::{Locale, ToFormattedString};

fn main() {
    println!("Hello, world!");
    let branchCreated = BranchCreated {
        name: "main".to_string(),
        country: "Japan".to_string(),
    };
    // Debug出力
    println!("{:?}", branchCreated);


    let partition_keys = PartitionKeys::from_aggregate_id(Uuid::new_v4());
    let event = Event {
        payload: branchCreated,
        partition_keys: partition_keys.clone(),
        sortable_unique_id: "unique_id_example".to_string(),
        version: 1,
    };

    // payloadを参照してDebug出力
    println!("Event Payload: {:?}", event);

    let branch = Branch {
        name: "main".to_string(),
        country: "Japan".to_string(),
    };

    let aggregate = Aggregate {
        payload: Box::new(branch),
        partition_keys: PartitionKeys {
            aggregate_id: uuid::Uuid::new_v4(),
            group_: "default".to_string(),
            root_partition_key: "default".to_string(),
        },
        version: 1,
        last_sortable_unique_id: "some_unique_id".to_string(),
    };
    println!("Aggregate Payload: {:?}", aggregate);

    let projector = BranchProjector {};
    let change_event_payload = BranchNameChanged {
        name: "main2".to_string(),
    };
    let event2 = Event {
        payload: change_event_payload,
        partition_keys : partition_keys.clone(),
        sortable_unique_id: "unique_id_example".to_string(),
        version: 2,
    };

    let new_payload = projector.project(aggregate.payload.as_ref(), &event2);

    println!("Aggregate Payload: (v2) {:?}", new_payload);

    let aggregateProjected = aggregate.project(&event2, &projector);

    println!("Aggregate Payload: (v2) {:?}", aggregateProjected);


    let timestamp = SystemTime::now();
    let uuid = Uuid::new_v4();

    let suid = SortableUniqueIdValue::generate(timestamp, uuid);
    println!("Generated SortableUniqueIdValue: {}", suid.0);

    let value1 = SortableUniqueIdValue::new(&suid.0);
    let value2 = SortableUniqueIdValue::get_current_id_from_utc();

    println!("Is value1 earlier than value2? {}", value1.is_earlier_than(&value2));
    let longtick = SortableUniqueIdValue::system_time_to_csharp_ticks(SystemTime::now());
    println!("Long tick: {}", longtick.to_formatted_string(&Locale::en));



    let mut repo = Repository::new();
    // Repositoryにイベントを保存
    let _ = repo.save(vec![Box::new(event)]);

    // 保存されているか確認するために再度ロードしてみる
    let loaded_aggregate = repo.load(
        &partition_keys,
        projector.clone()
    );

    println!("Loaded aggregate: {:?}", loaded_aggregate);

    // Event<BranchNameChanged> のイベントを追加する
    let _ = repo.save(vec![Box::new(event2)]);

    let loaded_aggregate = repo.load(
        &partition_keys,
        projector
    );
    println!("Loaded aggregate after name changed: {:?}", loaded_aggregate);


}
