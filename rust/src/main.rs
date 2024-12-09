mod simple;

use uuid::Uuid;
use simple::BranchCreated;
use crate::simple::{Aggregate, AggregateProjector, Branch, BranchNameChanged, BranchProjector, Event, PartitionKeys};

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
        version: 1,
    };

    let new_payload = projector.project(aggregate.payload.as_ref(), &event2);

    println!("Aggregate Payload: (v2) {:?}", new_payload);
}
