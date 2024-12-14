package domain

import (
	"fmt"
	"github.com/google/uuid"
	"math"
	"sort"
	"strconv"
	"strings"
	"time"
)

type EventPayload interface {
	IsEventPayload() bool
}

type PartitionKeys struct {
	AggregateID      uuid.UUID
	Group            string
	RootPartitionKey string
}
type EventCommon struct {
	Version          int64
	SortableUniqueID string
	PartitionKeys    PartitionKeys
	Payload          EventPayload
}

type AggregatePayload interface {
	IsAggregatePayload() bool
}
type EmptyAggregatePayload struct{}

func (e EmptyAggregatePayload) IsAggregatePayload() bool { return true }

type Aggregate struct {
	Payload              AggregatePayload
	PartitionKeys        PartitionKeys
	Version              int64
	LastSortableUniqueID string
}

type AggregateProjector interface {
	Project(payload AggregatePayload, ev *EventCommon) AggregatePayload
	GetVersion() string
}

func EmptyFromPartitionKeys(partitionKeys PartitionKeys) Aggregate {
	return Aggregate{
		Payload:              EmptyAggregatePayload{},
		PartitionKeys:        partitionKeys,
		Version:              0,
		LastSortableUniqueID: "",
	}
}

func (a Aggregate) Project(ev *EventCommon, projector AggregateProjector) Aggregate {
	return Aggregate{
		Payload:              projector.Project(a.Payload, ev),
		PartitionKeys:        a.PartitionKeys,
		Version:              ev.Version,
		LastSortableUniqueID: ev.SortableUniqueID,
	}
}

func (a Aggregate) ProjectAll(events []EventCommon, projector AggregateProjector) Aggregate {
	updated := a
	for _, ev := range events {
		updated = updated.Project(&ev, projector)
	}
	return updated
}

type SortableUniqueIdValue struct {
	Value string
}

const (
	SafeMilliseconds   = 5000
	TickNumberOfLength = 19
	IdNumberOfLength   = 11
	TickFormatter      = "%019d"
	IdFormatter        = "%011d"
)

var IdModBase = int64(math.Pow(10, IdNumberOfLength))

func NewSortableUniqueIdValue(value string) SortableUniqueIdValue {
	return SortableUniqueIdValue{Value: value}
}

func (s SortableUniqueIdValue) GetTicks() time.Time {
	ticksString := s.Value[:TickNumberOfLength]
	ticks, _ := strconv.ParseInt(ticksString, 10, 64)
	return time.Unix(0, ticks*100)
}

func (s SortableUniqueIdValue) GetSafeId() string {
	safeTicks := s.GetTicks().Add(-SafeMilliseconds*time.Millisecond).UnixNano() / 100
	return fmt.Sprintf(TickFormatter, safeTicks) + GetIdString(uuid.Nil)
}

func (s SortableUniqueIdValue) IsEarlierThan(toCompare SortableUniqueIdValue) bool {
	return strings.Compare(s.Value, toCompare.Value) < 0
}

func (s SortableUniqueIdValue) IsEarlierThanOrEqual(toCompare SortableUniqueIdValue) bool {
	return strings.Compare(s.Value, toCompare.Value) <= 0
}

func (s SortableUniqueIdValue) IsLaterThan(toCompare SortableUniqueIdValue) bool {
	return strings.Compare(s.Value, toCompare.Value) > 0
}

func (s SortableUniqueIdValue) IsLaterThanOrEqual(toCompare SortableUniqueIdValue) bool {
	return strings.Compare(s.Value, toCompare.Value) >= 0
}

func GenerateSortableUniqueID(timestamp time.Time, id uuid.UUID) string {
	return GetTickString(timestamp) + GetIdString(id)
}

func GetSafeIdFromUtc() string {
	return GetTickString(time.Now().UTC()) + GetIdString(uuid.Nil)
}

func GetCurrentIdFromUtc() string {
	return GetTickString(time.Now().UTC()) + GetIdString(uuid.New())
}

const (
	TicksPerSecond        = 10_000_000              // 1秒あたりのC#ティック数 (100ナノ秒単位)
	TicksFromUnixToCSharp = 621_355_968_000_000_000 // UNIX_EPOCHから1/1/0001までのティック数
)

func GetTickString(timestamp time.Time) string {
	ticks := systemTimeToCSharpTicks(timestamp)
	return fmt.Sprintf("%019d", ticks)
}

func systemTimeToCSharpTicks(timestamp time.Time) uint64 {
	// 現在の時間をUNIXエポックからの経過時間に変換
	durationSinceUnix := timestamp.Sub(time.Unix(0, 0))

	// 秒をティックに変換
	ticksSinceUnix := uint64(durationSinceUnix.Seconds()) * TicksPerSecond

	// ナノ秒をティックに変換（100ナノ秒単位）
	ticksSinceUnix += uint64(durationSinceUnix.Nanoseconds()%1_000_000_000) / 100

	// UNIX_EPOCHからC#エポックに合わせてオフセットを追加
	return ticksSinceUnix + TicksFromUnixToCSharp
}

func GetIdString(id uuid.UUID) string {
	hash := int64(id.ID())
	for _, r := range id {
		hash = hash*31 + int64(r)
	}
	return fmt.Sprintf(IdFormatter, int64(math.Abs(float64(hash))/float64(IdModBase)))
}

func NullableValue(value *string) *SortableUniqueIdValue {
	if value != nil {
		val := NewSortableUniqueIdValue(*value)
		return &val
	}
	return nil
}

func OptionalValue(value *string) *SortableUniqueIdValue {
	if value != nil && strings.TrimSpace(*value) != "" {
		val := NewSortableUniqueIdValue(*value)
		return &val
	}
	return nil
}

type Repository struct {
	Events []EventCommon
}

func NewRepository() *Repository {
	return &Repository{Events: make([]EventCommon, 0)}
}

// Load filters and projects events into an Aggregate based on the partition keys.
func (r *Repository) Load(partitionKeys PartitionKeys, projector AggregateProjector) (Aggregate, error) {
	// Filter events based on partition keys
	filtered := []EventCommon{}
	for _, ev := range r.Events {
		if ev.PartitionKeys == partitionKeys {
			filtered = append(filtered, ev)
		}
	}

	// Sort events by SortableUniqueId
	sort.Slice(filtered, func(i, j int) bool {
		return filtered[i].SortableUniqueID < filtered[j].SortableUniqueID
	})

	// Project events into an Aggregate
	aggregate := EmptyFromPartitionKeys(partitionKeys)
	projected := aggregate.ProjectAll(filtered, projector)

	return projected, nil
}

// Save adds a single event to the repository.
func (r *Repository) Save(newEvent EventCommon) error {
	r.Events = append(r.Events, newEvent)
	return nil
}

// Save adds multiple events to the repository.
func (r *Repository) SaveAll(newEvents []EventCommon) error {
	r.Events = append(r.Events, newEvents...)
	return nil
}

type CommandResponse struct {
	PartitionKeys PartitionKeys `json:"partition_keys"`
	Events        []EventCommon `json:"events"`
	Version       int64         `json:"version"`
}

type EventPayloadOrNone struct {
	HasValue bool
	Value    EventPayload
}

func ReturnEventPayload(value EventPayload) EventPayloadOrNone {
	return EventPayloadOrNone{HasValue: true, Value: value}
}

func (e EventPayloadOrNone) GetValue() EventPayload {
	return e.Value
}

type CommandContext struct {
	Aggregate Aggregate
	Projector AggregateProjector
	Events    []EventCommon
}

func (ctx *CommandContext) AppendEvent(eventPayload EventPayload) EventPayloadOrNone {
	// TypedEventを生成
	toAdd := EventCommon{
		Version:          ctx.Aggregate.Version + 1,
		SortableUniqueID: GetCurrentIdFromUtc(),
		PartitionKeys:    ctx.Aggregate.PartitionKeys,
		Payload:          eventPayload,
	}
	aggregate := ctx.Aggregate.Project(&toAdd, ctx.Projector)
	ctx.Aggregate = aggregate
	ctx.Events = append(ctx.Events, toAdd)
	return EventPayloadOrNone{HasValue: false}
}

type Command interface {
	IsCommand() bool
}

func ExecuteCommandWithHandler[TCommand CommandWithHandler](repository *Repository, command TCommand) (CommandResponse, error) {
	return ExecuteCommand(repository,
		command,
		command.GetProjector(),
		func(command TCommand) PartitionKeys { return command.SpecifyPartitionKeys() },
		func(command TCommand, context CommandContext) EventPayloadOrNone {
			return command.Handle(context)
		})
}

func ExecuteCommand[TCommand Command](repository *Repository,
	command TCommand,
	projector AggregateProjector,
	partitionKeysProvider func(command TCommand) PartitionKeys,
	commandHandler func(command TCommand, context CommandContext) EventPayloadOrNone) (CommandResponse, error) {
	partitionKeys := partitionKeysProvider(command)

	currentAggregate, err := repository.Load(partitionKeys, projector)
	if err != nil {
		return CommandResponse{}, err
	}

	context := CommandContext{
		Events:    make([]EventCommon, 0),
		Aggregate: currentAggregate,
		Projector: projector,
	}

	// コマンドハンドラーを実行
	if eventPayloadOrNone := commandHandler(command, context); eventPayloadOrNone.HasValue {
		lastEvent := EventCommon{
			Payload:          eventPayloadOrNone.GetValue(),
			PartitionKeys:    currentAggregate.PartitionKeys,
			SortableUniqueID: GenerateSortableUniqueID(time.Now().UTC(), uuid.New()),
			Version:          currentAggregate.Version + 1,
		}
		context.Aggregate = context.Aggregate.Project(&lastEvent, projector)
		context.Events = append(context.Events, lastEvent)
	}
	currentAggregate = context.Aggregate
	// イベントを保存
	savedEvents := context.Events
	err = repository.SaveAll(savedEvents)
	if err != nil {
		fmt.Println("Error saving events:", err)
		return CommandResponse{}, err
	}

	// コマンドレスポンスを生成
	return CommandResponse{
		PartitionKeys: currentAggregate.PartitionKeys,
		Events:        savedEvents,
		Version:       currentAggregate.Version,
	}, nil
}

type CommandWithHandler interface {
	Command
	Handle(context CommandContext) EventPayloadOrNone
	SpecifyPartitionKeys() PartitionKeys
	GetProjector() AggregateProjector
}
