import { AggregatePayload, AggregateProjector, EmptyAggregatePayload, EventCommon, EventPayload, Command, CommandWithHandler, CommandContext, EventPayloadOrNone, PartitionKeys } from "./lib";

export class BranchCreated implements EventPayload {
    type: string = BranchCreated.name;
    constructor(public readonly name: string, public readonly country: string) {}
    IsEventPayload(): boolean {
        return true;
    }
}
export class BranchNameChanged implements EventPayload {
    type: string = BranchNameChanged.name;
    constructor(public readonly name: string) {}
    IsEventPayload(): boolean {
        return true;
    }
}
export class BranchCountryChanged implements EventPayload {
    type: string = BranchCountryChanged.name;
    constructor(public country: string) {}
    IsEventPayload(): boolean {
        return true;
    }
}

export class Branch implements AggregatePayload {
    type: string = 'Branch';
    constructor(public readonly name: string, public readonly country: string) {}
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
export class CreateBranch implements Command {
    IsCommand(): boolean {
        return true;
    }
    constructor(public readonly name: string, public readonly country: string) {}
}
export class ChangeBranchName implements CommandWithHandler<ChangeBranchName> {
    constructor(public readonly name: string, public readonly partitionKeys: PartitionKeys ) {}
    GetAggregateProjector(): AggregateProjector {return new BranchProjector();}
    PartitionKeysProvider(command: ChangeBranchName): PartitionKeys { return command.partitionKeys;}
    CommandHandler(command: ChangeBranchName, context: CommandContext): EventPayloadOrNone {
        return new BranchNameChanged(command.name);
    }
    IsCommand(): boolean { return true;}
}
export class ChangeBranchCountry implements CommandWithHandler<ChangeBranchCountry> {
    constructor(public readonly country: string, public readonly partitionKeys: PartitionKeys ) {}
    GetAggregateProjector(): AggregateProjector {return new BranchProjector();}
    PartitionKeysProvider(command: ChangeBranchCountry): PartitionKeys { return command.partitionKeys;}
    CommandHandler(command: ChangeBranchCountry, context: CommandContext): EventPayloadOrNone {
        return new BranchCountryChanged(command.country);
    }
    IsCommand(): boolean { return true;}
}