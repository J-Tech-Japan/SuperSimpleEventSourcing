import { v4 as uuidv4 } from 'uuid';

export interface EventPayload {
    IsEventPayload() : boolean;
}

export class PartitionKeys {
  constructor(public readonly AggregateID: string, public readonly Group: string, public readonly RootPartitionKey: string) {}
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
    public static EmptyFromPartitionKeys(partitionKeys: PartitionKeys): Aggregate {
        return new Aggregate(new EmptyAggregatePayload(), partitionKeys, 0, '');
    }
}

export interface AggregateProjector {
    Project(payload: AggregatePayload, ev: EventCommon): AggregatePayload;
    GetVersion(): string;
}


const SafeMilliseconds = 5000;
const TickNumberOfLength = 19;
const IdNumberOfLength = 11;
const TickFormatter = "%019d";
const IdFormatter = "%011d";
const IdModBase = Math.pow(10, IdNumberOfLength);

export class SortableUniqueIdValue {
    
  constructor(public value: string) {}

  // GetTicks
  getTicks(): Date {
    const ticksString = this.value.slice(0, TickNumberOfLength);
    const ticks = BigInt(ticksString);
    return new Date(Number(ticks) / 1000000);
  }

  getSafeId(): string {
    const safeTicks = BigInt(this.getTicks().getTime()) - BigInt(SafeMilliseconds);
    return `${SortableUniqueIdValue.formatTick(safeTicks)}${this.getIdString('00000000-0000-0000-0000-000000000000')}`;
  }

  // Comparison Methods
  isEarlierThan(toCompare: SortableUniqueIdValue): boolean {
    return this.value < toCompare.value;
  }

  isEarlierThanOrEqual(toCompare: SortableUniqueIdValue): boolean {
    return this.value <= toCompare.value;
  }

  isLaterThan(toCompare: SortableUniqueIdValue): boolean {
    return this.value > toCompare.value;
  }

  isLaterThanOrEqual(toCompare: SortableUniqueIdValue): boolean {
    return this.value >= toCompare.value;
  }

  // Helper Methods
  private static formatTick(ticks: BigInt): string {
    return ticks.toString().padStart(TickNumberOfLength, '0');
  }

  private getIdString(id: string): string {
    const hash = this.generateIdHash(id);
    return this.formatId(hash);
  }

  private generateIdHash(id: string): number {
    let hash = 0;
    for (let i = 0; i < id.length; i++) {
      hash = (31 * hash + id.charCodeAt(i)) & 0xffffffff;
    }
    return Math.abs(hash);
  }

  private formatId(hash: number): string {
    return (hash % IdModBase).toString().padStart(IdNumberOfLength, '0');
  }

  // Static Methods
  static generateSortableUniqueID(timestamp: Date, id: string): string {
    return SortableUniqueIdValue.getTickString(timestamp) + SortableUniqueIdValue.getIdString(id);
  }

  static getSafeIdFromUtc(): string {
    return SortableUniqueIdValue.getTickString(new Date()) + SortableUniqueIdValue.getIdString('00000000-0000-0000-0000-000000000000');
  }

  static getCurrentIdFromUtc(): string {
    return SortableUniqueIdValue.getTickString(new Date()) + SortableUniqueIdValue.getIdString(SortableUniqueIdValue.generateUuid());
  }

  static getTickString(timestamp: Date): string {
    const ticks = SortableUniqueIdValue.systemTimeToCSharpTicks(timestamp);
    return SortableUniqueIdValue.formatTick(BigInt(ticks));
  }

  static systemTimeToCSharpTicks(timestamp: Date): number {
    const durationSinceUnix = timestamp.getTime() - Date.UTC(1970, 0, 1);
    const ticksSinceUnix = Math.floor(durationSinceUnix * 10000);
    return ticksSinceUnix + TicksFromUnixToCSharp;
  }

  static getIdString(id: string): string {
    const hash = SortableUniqueIdValue.generateIdHash(id);
    return SortableUniqueIdValue.formatId(hash);
  }

  static generateUuid(): string {
    return uuidv4();
  }

  static generateIdHash(id: string): number {
    let hash = 0;
    for (let i = 0; i < id.length; i++) {
      hash = (31 * hash + id.charCodeAt(i)) & 0xffffffff;
    }
    return Math.abs(hash);
  }

  static formatId(hash: number): string {
    return (hash % IdModBase).toString().padStart(IdNumberOfLength, '0');
  }

  // Nullable and Optional Value handlers
  static nullableValue(value: string | null): SortableUniqueIdValue | null {
    if (value !== null) {
      return new SortableUniqueIdValue(value);
    }
    return null;
  }

  static optionalValue(value: string | null): SortableUniqueIdValue | null {
    if (value !== null && value.trim() !== "") {
      return new SortableUniqueIdValue(value);
    }
    return null;
  }
}

const TicksPerSecond = 10_000_000;
const TicksFromUnixToCSharp = 621_355_968_000_000_000;

export class Repository {
  private Events: EventCommon[];

  constructor() {
      this.Events = [];
  }

  // Load filters and projects events into an Aggregate based on the partition keys.
  Load(partitionKeys: PartitionKeys, projector: AggregateProjector): Aggregate {
      // Filter events based on partition keys
      const filtered = this.Events.filter(ev => ev.PartitionKeys === partitionKeys);

      // Sort events by SortableUniqueID
      filtered.sort((a, b) => {
          if (a.SortableUniqueID < b.SortableUniqueID) return -1;
          if (a.SortableUniqueID > b.SortableUniqueID) return 1;
          return 0;
      });

      // Project events into an Aggregate
      const aggregate = Aggregate.EmptyFromPartitionKeys(partitionKeys);
      const projected = aggregate.ProjectAll(filtered, projector);

      return projected;
  }

  // Save adds a single event to the repository.
  Save(newEvent: EventCommon): void {
      this.Events.push(newEvent);
  }

  // SaveAll adds multiple events to the repository.
  SaveAll(newEvents: EventCommon[]): void {
      this.Events.push(...newEvents);
  }
}

export type CommandResponse = {
  partitionKeys: PartitionKeys;
  events: EventCommon[];
  version: number;
};


export type None = {};
export const None : None = {};
export type EventPayloadOrNone = EventPayload | None;

export class CommandContext {
  Aggregate: Aggregate;
  Projector: AggregateProjector;
  Events: EventCommon[];

  constructor(aggregate: Aggregate, projector: AggregateProjector, events: EventCommon[] = []) {
      this.Aggregate = aggregate;
      this.Projector = projector;
      this.Events = events;
  }

  AppendEvent(eventPayload: EventPayload): EventPayloadOrNone {
      const toAdd: EventCommon = {
          Version: this.Aggregate.Version + 1,
          SortableUniqueID: SortableUniqueIdValue.getCurrentIdFromUtc(),
          PartitionKeys: this.Aggregate.PartitionKeys,
          Payload: eventPayload
      };

      const aggregate = this.Aggregate.Project(toAdd, this.Projector);
      this.Aggregate = aggregate;
      this.Events.push(toAdd);

      return None;
  }
}

export interface Command {
  IsCommand(): boolean;
}

export interface CommandWithHandler<TCommand extends Command> extends Command {
  GetAggregateProjector(): AggregateProjector;
  PartitionKeysProvider(command: TCommand): PartitionKeys;
  CommandHandler(command: TCommand, context: CommandContext): EventPayloadOrNone;
}


export class CommandExecutor {
  private Repository: Repository;

  constructor(repository: Repository) {
      this.Repository = repository;
  }

  public ExecuteCommandWithHandler<TCommand extends CommandWithHandler<TCommand>>(
    command: TCommand
  ): CommandResponse {
      return this.ExecuteCommand(
          command,
          command.GetAggregateProjector(),
          command.PartitionKeysProvider,
          command.CommandHandler
      );
  }

  public ExecuteCommand<TCommand extends Command>(
    command: TCommand,
    projector: AggregateProjector,
    partitionKeysProvider: (command: TCommand) => PartitionKeys,
    commandHandler: (command: TCommand, context: CommandContext) => EventPayloadOrNone
  ): CommandResponse {
      const partitionKeys = partitionKeysProvider(command);
      const currentAggregate = this.Repository.Load(partitionKeys, projector);
      const context = new CommandContext(currentAggregate, projector, []);
      const eventPayloadOrNone = commandHandler(command, context);

      if (eventPayloadOrNone !== None) {
          let eventPayload: EventPayload = eventPayloadOrNone as EventPayload;
          const lastEvent: EventCommon = {
              Payload: eventPayload,
              PartitionKeys: currentAggregate.PartitionKeys,
              SortableUniqueID: SortableUniqueIdValue.getCurrentIdFromUtc(),
              Version: currentAggregate.Version + 1,
          };

          context.Aggregate = context.Aggregate.Project(lastEvent, projector);
          context.Events.push(lastEvent);
      }
      const updatedAggregate = context.Aggregate;
      this.Repository.SaveAll(context.Events);
      return {
          partitionKeys: updatedAggregate.PartitionKeys,
          events: context.Events,
          version: updatedAggregate.Version,
      };
  }
}