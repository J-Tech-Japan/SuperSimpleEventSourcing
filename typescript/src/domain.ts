import { AggregatePayload, AggregateProjector, EmptyAggregatePayload, EventCommon, EventPayload } from "./lib";

export class BranchCreated implements EventPayload {
    type: string = 'BranchCreated';
    constructor(public name: string, public country: string) {}
    IsEventPayload(): boolean {
        return true;
    }
}
export class BranchNameChanged implements EventPayload {
    type: string = 'BranchNameChanged';
    constructor(public name: string) {}
    IsEventPayload(): boolean {
        return true;
    }
}
export class BranchCountryChanged implements EventPayload {
    type: string = 'BranchCountryChanged';
    constructor(public country: string) {}
    IsEventPayload(): boolean {
        return true;
    }
}

export class Branch implements AggregatePayload {
    type: string = 'Branch';
    constructor(public name: string, public country: string) {}
    IsAggregatePayload() : boolean {
        return true;
    }
}

export class BranchProjector implements AggregateProjector {
    constructor() {}
    Project(payload: AggregatePayload, ev: EventCommon): AggregatePayload {
        if (payload instanceof Branch) {
            if (ev.Payload instanceof BranchNameChanged) {
                return new Branch(ev.Payload.name, payload.country);
            } else if (ev.Payload instanceof BranchCountryChanged) {
                return new Branch(payload.name, ev.Payload.country);
            }
        } else if (payload instanceof EmptyAggregatePayload && ev.Payload instanceof BranchCreated) {
            return new Branch(ev.Payload.name, ev.Payload.country);
        }
        return payload;
    }
    GetVersion(): string { return '1.0.0'; }

}