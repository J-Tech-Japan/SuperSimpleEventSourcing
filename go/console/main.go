package main

import (
	"fmt"
	"github.com/google/uuid"
	"go_eventsourcing/domain"
	"time"
)

func main() {
	//var branchCreated = domain.BranchCreated{
	//	Name: "Tokyo",
	//	Country: "Jap" +
	//		"an",
	//} // Print the entire struct with field names and values
	//fmt.Printf("Branch Details2: %+v\n", branchCreated)
	//
	//pk := domain.PartitionKeys{
	//	AggregateID:      uuid.New(),
	//	Group:            "default",
	//	RootPartitionKey: "default",
	//}
	//fmt.Printf("Partition Key: %+v\n", pk)
	//
	//event := domain.EventCommon{
	//	Version:          1,
	//	SortableUniqueID: domain.GetCurrentIdFromUtc(),
	//	PartitionKeys:    pk,
	//	Payload:          branchCreated,
	//}
	//fmt.Printf("event: %+v\n", event)
	//emptyPayload := domain.EmptyAggregatePayload{}
	//branchProjector := domain.BranchProjector{}
	//createdPayload := branchProjector.Project(emptyPayload, &event)
	//fmt.Printf("created payload: %+v\n", createdPayload)
	//changedName := domain.BranchNameChanged{Name: "Osaka"}
	//event2 := domain.EventCommon{
	//	Version:          2,
	//	SortableUniqueID: domain.GetCurrentIdFromUtc(),
	//	PartitionKeys:    pk,
	//	Payload:          changedName,
	//}
	//changedNamePayload := branchProjector.Project(createdPayload, &event2)
	//fmt.Printf("changed name payload: %+v\n", changedNamePayload)
	//
	//aggregateStart := domain.EmptyFromPartitionKeys(pk)
	//fmt.Printf("aggregate start: %+v\n", aggregateStart)
	//aggregateAfterCreated := aggregateStart.Project(&event, branchProjector)
	//fmt.Printf("branch created aggregate: %+v\n", aggregateAfterCreated)
	//
	sortableUniqueId := domain.GenerateSortableUniqueID(time.Now().UTC(), uuid.New())
	fmt.Printf("sortable unique ID: %+v\n", sortableUniqueId)

	timsstamp := time.Now().UTC()
	durationSinceUnix := timsstamp.Sub(time.Unix(0, 0))
	ticksSinceUnix := uint64(durationSinceUnix.Seconds()) * 10_000_000
	fmt.Printf("duration since unix: %+v\n", durationSinceUnix)
	fmt.Printf("ticks since unix: %+v\n", ticksSinceUnix)
	//
	//repository := domain.NewRepository()
	//if err := repository.Save(event); err != nil {
	//	fmt.Printf("Error saving event: %v\n", err)
	//}
	//if err := repository.Save(event2); err != nil {
	//	fmt.Printf("Error saving event2: %v\n", err)
	//}
	//aggregate, err := repository.Load(pk, branchProjector)
	//if err != nil {
	//	fmt.Printf("Error loading aggregate: %v\n", err)
	//}
	//fmt.Printf("aggregate: %+v\n", aggregate)
	repository := domain.NewRepository()
	createBranch := domain.CreateBranchCommand{
		Name:    "Tokyo",
		Country: "Japan",
	}
	branchProjector := domain.BranchProjector{}
	// command and method to handle
	response, err := domain.ExecuteCommand(repository,
		createBranch,
		branchProjector,
		func(command domain.CreateBranchCommand) domain.PartitionKeys {
			return domain.PartitionKeys{
				AggregateID:      uuid.New(),
				Group:            "default",
				RootPartitionKey: "default",
			}
		},
		func(command domain.CreateBranchCommand, context domain.CommandContext) domain.EventPayloadOrNone {
			return domain.ReturnEventPayload(domain.BranchCreated{command.Name, command.Country})
		})
	if err != nil {
		fmt.Printf("Error executing command: %v\n", err)
	}
	changeNameCommand := domain.ChangeBranchNameCommand{
		Name:          "Osaka",
		PartitionKeys: response.PartitionKeys,
	}
	// command and method to handle
	response, err = domain.ExecuteCommand(repository,
		changeNameCommand,
		branchProjector,
		func(command domain.ChangeBranchNameCommand) domain.PartitionKeys {
			return command.PartitionKeys
		},
		func(command domain.ChangeBranchNameCommand, context domain.CommandContext) domain.EventPayloadOrNone {
			return domain.ReturnEventPayload(domain.BranchNameChanged{command.Name})
		})
	if err != nil {
		fmt.Printf("Error executing command: %v\n", err)
	}
	changeCountryCommand := domain.ChangeBranchCountryCommand{
		Country:       "USA",
		PartitionKeys: response.PartitionKeys,
	}
	// command with Handler includes the command and method to handle
	response, err = domain.ExecuteCommandWithHandler(repository, changeCountryCommand)
	if err != nil {
		fmt.Printf("Error executing command: %v\n", err)
	}
	aggregate3, err := repository.Load(response.PartitionKeys, branchProjector)
	fmt.Printf("aggregate: %+v\n", aggregate3)
}
