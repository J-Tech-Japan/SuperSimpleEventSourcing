// hello world
console.log('hello world');

import { BranchCreated, BranchProjector } from './domain';
import { PartitionKeys, EventCommon, EmptyAggregatePayload, SortableUniqueIdValue } from './lib';
import { v4 as uuidv4 } from 'uuid';

const branchCreatedEventPaylaod = new BranchCreated('London', 'UK');
console.log(branchCreatedEventPaylaod);

const partitionKey : PartitionKeys = {
    AggregateID: uuidv4(),
    Group: 'Branch',
    RootPartitionKey: 'Branch',
};
console.log(partitionKey);

const branchCreatedEvent : EventCommon = {
    Version: 1,
    SortableUniqueID: '1',
    PartitionKeys: partitionKey,
    Payload: branchCreatedEventPaylaod,
};
console.log(branchCreatedEvent);

const branchProjector = new BranchProjector();
console.log(branchProjector.GetVersion());
const branch = branchProjector.Project(new EmptyAggregatePayload(), branchCreatedEvent);

console.log(branch);

const SortableUniqueID = SortableUniqueIdValue.generateSortableUniqueID(new Date(), uuidv4());
console.log(SortableUniqueID);
