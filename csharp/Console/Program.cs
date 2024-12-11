// See https://aka.ms/new-console-template for more information

using System.Text.Json;
using Domain;
using ResultBoxes;

var executor = new CommandExecutor { EventTypes = new DomainEventTypes() };

foreach (var arg in args)
{
    var response = await executor.Execute(new RegisterBranch(arg)).UnwrapBox();
    var aggregate = Repository.Load<BranchProjector>(response.PartitionKeys).UnwrapBox();
    Console.WriteLine(JsonSerializer.Serialize(aggregate.ToTypedPayload<Branch>().UnwrapBox()));
}

var generatedSortableUniqueId = SortableUniqueIdValue.Generate(SekibanDateProducer.GetRegistered().UtcNow, Guid.NewGuid());
Console.WriteLine($"Generated SortableUniqueId: {generatedSortableUniqueId}");
Console.WriteLine($"tick value: {DateTime.UtcNow.Ticks:N0}");


Console.WriteLine("input new branch name:");
var inputN = Console.ReadLine();
var responseN = await executor.Execute(new RegisterBranch(inputN)).UnwrapBox();
var aggregateN = Repository.Load<BranchProjector>(responseN.PartitionKeys).UnwrapBox();
Console.WriteLine(JsonSerializer.Serialize(aggregateN.ToTypedPayload<Branch>().UnwrapBox()));



while (true)
{
    Console.WriteLine("ChangeName: input changing name (if you want to stop, type 'exit':");
    var input = Console.ReadLine();

    if (input?.ToLower() == "exit")
    {
        break;
    }

    if (!string.IsNullOrEmpty(input))
    {
        var response = await executor.Execute(new ChangeBranchName(responseN.PartitionKeys.AggregateId, input??"")).UnwrapBox();
        var aggregate = Repository.Load<BranchProjector>(response.PartitionKeys).UnwrapBox();
        Console.WriteLine(JsonSerializer.Serialize(aggregate.ToTypedPayload<Branch>().UnwrapBox()));
    }
}