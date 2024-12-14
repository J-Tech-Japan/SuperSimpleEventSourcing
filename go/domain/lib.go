package domain

import (
	"fmt"
	"github.com/google/uuid"
	"math"
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
	now := time.Now().UTC().UnixNano() / 100
	return fmt.Sprintf(TickFormatter, now) + GetIdString(uuid.New())
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
