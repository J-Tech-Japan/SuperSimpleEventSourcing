// export type BranchCreated = EventPayload<{ 
//     type: 'BranchCreated';
//     name: string;
//     country: string;
// }>;
class BranchCreated implements EventPayload {
    type: string = 'BranchCreated';
    constructor(public name: string, public country: string) {}
    IsEventPayload(): boolean {
        return true;
    }
}
export { BranchCreated };