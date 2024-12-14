package main

import (
	"fmt"
	"github.com/google/uuid"
	"time"
)
import "go_eventsourcing/domain"

func main() {
	var branchCreated = domain.BranchCreated{
		Name: "Tokyo",
		Country: "Jap" +
			"an",
	} // Print the entire struct with field names and values
	fmt.Printf("Branch Details2: %+v\n", branchCreated)

	pk := domain.PartitionKeys{
		AggregateID:      uuid.New(),
		Group:            "default",
		RootPartitionKey: "default",
	}
	fmt.Printf("Partition Key: %+v\n", pk)

	event := domain.EventCommon{
		Version:          1,
		SortableUniqueID: domain.GetCurrentIdFromUtc(),
		PartitionKeys:    pk,
		Payload:          branchCreated,
	}
	fmt.Printf("event: %+v\n", event)
	emptyPayload := domain.EmptyAggregatePayload{}
	branchProjector := domain.BranchProjector{}
	createdPayload := branchProjector.Project(emptyPayload, &event)
	fmt.Printf("created payload: %+v\n", createdPayload)
	changedName := domain.BranchNameChanged{Name: "Osaka"}
	event2 := domain.EventCommon{
		Version:          2,
		SortableUniqueID: domain.GetCurrentIdFromUtc(),
		PartitionKeys:    pk,
		Payload:          changedName,
	}
	changedNamePayload := branchProjector.Project(createdPayload, &event2)
	fmt.Printf("changed name payload: %+v\n", changedNamePayload)

	aggregateStart := domain.EmptyFromPartitionKeys(pk)
	fmt.Printf("aggregate start: %+v\n", aggregateStart)
	aggregateAfterCreated := aggregateStart.Project(&event, branchProjector)
	fmt.Printf("branch created aggregate: %+v\n", aggregateAfterCreated)

	sortableUniqueId := domain.GenerateSortableUniqueID(time.Now().UTC(), uuid.New())
	fmt.Printf("sortable unique ID: %+v\n", sortableUniqueId)

	repository := domain.NewRepository()
	if err := repository.Save(event); err != nil {
		fmt.Printf("Error saving event: %v\n", err)
	}
	if err := repository.Save(event2); err != nil {
		fmt.Printf("Error saving event2: %v\n", err)
	}
	aggregate, err := repository.Load(pk, branchProjector)
	if err != nil {
		fmt.Printf("Error loading aggregate: %v\n", err)
	}
	fmt.Printf("aggregate: %+v\n", aggregate)

}
