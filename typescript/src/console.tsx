// hello world
console.log('hello world');

import { BranchCreated, BranchNameChanged, BranchProjector, ChangeBranchCountry, ChangeBranchName, CreateBranch } from './domain';
import { PartitionKeys, EventCommon, EmptyAggregatePayload, SortableUniqueIdValue, Repository, CommandExecutor } from './lib';
import { v4 as uuidv4 } from 'uuid';

// const branchCreatedEventPaylaod = new BranchCreated('London', 'UK');
// console.log(branchCreatedEventPaylaod);

// const partitionKey : PartitionKeys = {
//     AggregateID: uuidv4(),
//     Group: 'Branch',
//     RootPartitionKey: 'Branch',
// };
// console.log(partitionKey);

// const branchCreatedEvent : EventCommon = {
//     Version: 1,
//     SortableUniqueID: '1',
//     PartitionKeys: partitionKey,
//     Payload: branchCreatedEventPaylaod,
// };
// console.log(branchCreatedEvent);

// const branchProjector = new BranchProjector();
// console.log(branchProjector.GetVersion());
// const branch = branchProjector.Project(new EmptyAggregatePayload(), branchCreatedEvent);

// console.log(branch);

// const SortableUniqueID = SortableUniqueIdValue.generateSortableUniqueID(new Date(), uuidv4());
// console.log(SortableUniqueID);

// const repository = new Repository();
// var event1 : EventCommon = {
//     Version: 1,
//     SortableUniqueID: SortableUniqueIdValue.generateSortableUniqueID(new Date(), uuidv4()),
//     PartitionKeys: partitionKey,
//     Payload: new BranchCreated('London', 'UK'),
// };
// repository.Save(event1);
// var event2 : EventCommon = {
//     Version: 2,
//     SortableUniqueID: SortableUniqueIdValue.generateSortableUniqueID(new Date(), uuidv4()),
//     PartitionKeys: partitionKey,
//     Payload: new BranchNameChanged('Manchester'),
// };
// repository.Save(event2);

// var aggregate = repository.Load(partitionKey, new BranchProjector());
// console.log(aggregate);


const repository = new Repository();
const commandExecutor = new CommandExecutor(repository);

const createBranchCommand = new CreateBranch('London', 'UK');
let commandResponse = commandExecutor.ExecuteCommand(
    createBranchCommand, 
    new BranchProjector(), 
    (command) => new PartitionKeys(uuidv4(), 'Branch', 'Branch'),
    (command, context) => { return new BranchCreated(command.name, command.country);});

const aggregate2 = repository.Load(commandResponse.partitionKeys, new BranchProjector());
console.log(aggregate2);

const changeBranchNameCommand = new ChangeBranchName('Manchester', commandResponse.partitionKeys);

// commandResponse = commandExecutor.ExecuteCommand(
//     changeBranchNameCommand, 
//     changeBranchNameCommand.GetAggregateProjector(), 
//     changeBranchNameCommand.PartitionKeysProvider, 
//     changeBranchNameCommand.CommandHandler);
commandResponse = commandExecutor.ExecuteCommandWithHandler(changeBranchNameCommand);
commandResponse = commandExecutor.ExecuteCommandWithHandler(new ChangeBranchCountry('England', commandResponse.partitionKeys));

const aggregate3 = repository.Load(commandResponse.partitionKeys, new BranchProjector());
console.log(aggregate3);
