mod simple;

use uuid::Uuid;
use simple::BranchCreated;
use crate::simple::{AggregateProjector, BranchNameChanged, BranchProjector, ChangeBranchCountryNameCommand, ChangeBranchNameCommand, CommandExecutor, CommandWithHandler, CreateBranchCommand, PartitionKeys, Repository};
use num_format::{ToFormattedString};

fn main() {

    let mut repo = Repository::new();
    let projector = BranchProjector {};

    let create_branch_command = CreateBranchCommand {
        name: "main".to_string(),
        country: "Japan".to_string(),
    };
    let mut command_executor = CommandExecutor {
        repository: repo,
    };
    let response = command_executor.execute(create_branch_command, &projector, |command| PartitionKeys {
        aggregate_id: Uuid::new_v4(),
        group_: "default".to_string(),
        root_partition_key: "default".to_string(),
    }, |command, context| Some(Box::new(BranchCreated {
        name: command.name.clone(),
        country: command.country.clone(),
    })));

    println!("Command executed: {:?}", response);
    let loaded_aggregate = command_executor.repository.load(
        &response.partition_keys,
        &projector
    );
    println!("Loaded Aggregate: {:?}", response);
    let change_branch_name_command = ChangeBranchNameCommand {
        name: "main2".to_string(),
        partition_keys: response.partition_keys.clone()
    };
    let response = command_executor.execute(change_branch_name_command,
                                            &projector,
                                            |command| command.partition_keys.clone(),
                                            |command, context| Some(Box::new(BranchNameChanged {
        name: command.name.clone(),
    })));
    println!("Change Name Command executed: {:?}", response);
    let loaded_aggregate = command_executor.repository.load(
        &response.partition_keys,
        &projector
    );
    println!("Loaded Aggregate After Name Change: {:?}", loaded_aggregate);

    let change_branch_country_name_command = ChangeBranchCountryNameCommand {
        country: "USA".to_string(),
        partition_keys: response.partition_keys.clone()
    };
    let response = command_executor.execute(change_branch_country_name_command.clone(),
                                            change_branch_country_name_command.get_projector().as_ref(),
                                            ChangeBranchCountryNameCommand::get_partition_keys,
                                            ChangeBranchCountryNameCommand::command_handler);

    println!("Change Name Change Country: {:?}", response);
    let loaded_aggregate = command_executor.repository.load(
        &response.partition_keys,
        &projector
    );
    println!("Loaded Aggregate After Change Country: {:?}", loaded_aggregate);

}
