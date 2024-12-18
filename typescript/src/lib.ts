export interface EventPayload {
    IsEventPayload() : boolean;
}

export type PartitionKeys = {
    AggregateID: string;  // UUID is represented as a string in TypeScript
    Group: string;
    RootPartitionKey: string;
}

export type EventCommon = {
    Version: number;
    SortableUniqueID: string;
    PartitionKeys: PartitionKeys;
    Payload: EventPayload;
}

export interface AggregatePayload {
    IsAggregatePayload(): boolean;
}

export class EmptyAggregatePayload implements AggregatePayload {
    IsAggregatePayload(): boolean {
        return true;
    }
}

export class Aggregate {
    Payload: AggregatePayload;
    PartitionKeys: PartitionKeys;
    Version: number;
    LastSortableUniqueID: string;

    constructor(payload: AggregatePayload, partitionKeys: PartitionKeys, version: number, lastSortableUniqueID: string) {
        this.Payload = payload;
        this.PartitionKeys = partitionKeys;
        this.Version = version;
        this.LastSortableUniqueID = lastSortableUniqueID;
    }
    Project(ev: EventCommon, projector: AggregateProjector): Aggregate {
        return new Aggregate(
            projector.Project(this.Payload, ev),  // Payload をプロジェクタで更新
            this.PartitionKeys,
            ev.Version,
            ev.SortableUniqueID
        );
    }
    ProjectAll(events: EventCommon[], projector: AggregateProjector): Aggregate {
        var updated : Aggregate = this;
        for (const ev of events) {
            updated = updated.Project(ev, projector);  // 各イベントで Project を呼び出して更新
        }
        return updated;
    }
}

export interface AggregateProjector {
    Project(payload: AggregatePayload, ev: EventCommon): AggregatePayload;
    GetVersion(): string;
}