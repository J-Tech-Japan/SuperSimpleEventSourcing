package domain

import "github.com/google/uuid"

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
