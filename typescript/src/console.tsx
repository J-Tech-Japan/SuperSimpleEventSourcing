// hello world
console.log('hello world');

import { BranchCreated } from './domain';

// const branch: BranchCreated = {
//     type: 'BranchCreated',
//     name: 'London',
//     country: 'UK',
//     IsEventPayload: () => true,
// };

const branch = new BranchCreated('London', 'UK');
console.log(branch);