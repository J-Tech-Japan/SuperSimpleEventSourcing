package domain

type BranchCreated struct {
	Name    string
	Country string
}

func (b BranchCreated) IsEventPayload() bool {
	return true
}

type BranchNameChanged struct {
	Name string
}

func (b BranchNameChanged) IsEventPayload() bool {
	return true
}

type BranchCountryChanged struct {
	Country string
}

func (b BranchCountryChanged) IsEventPayload() bool {
	return true
}

type Branch struct {
	Name    string
	Country string
}

func (b Branch) IsAggregatePayload() bool {
	return true
}

type BranchProjector struct{}

func (p BranchProjector) GetVersion() string {
	return "1.0"
}

func (p BranchProjector) Project(payload AggregatePayload, ev *EventCommon) AggregatePayload {
	switch ap := payload.(type) {
	case EmptyAggregatePayload:
		switch ep := ev.Payload.(type) {
		case BranchCreated:
			return Branch{Name: ep.Name, Country: ep.Country}
		default:
			return payload
		}
	case Branch:
		switch ep := ev.Payload.(type) {
		case BranchNameChanged:
			return Branch{
				Name:    ep.Name,
				Country: ap.Country,
			}
		case BranchCountryChanged:
			return Branch{
				Name:    ap.Name,
				Country: ep.Country,
			}
		default:
			return payload
		}
	default:
		return payload
	}
}

type CreateBranchCommand struct {
	Name    string
	Country string
}

func (c CreateBranchCommand) IsCommand() bool {
	return true
}

type ChangeBranchNameCommand struct {
	Name          string
	PartitionKeys PartitionKeys
}

func (c ChangeBranchNameCommand) IsCommand() bool {
	return true
}

type ChangeBranchCountryCommand struct {
	Country       string
	PartitionKeys PartitionKeys
}

func (c ChangeBranchCountryCommand) IsCommand() bool {
	return true
}

func (c ChangeBranchCountryCommand) Handle(context CommandContext) EventPayloadOrNone {
	return ReturnEventPayload(BranchCountryChanged{Country: c.Country})
}

func (c ChangeBranchCountryCommand) SpecifyPartitionKeys() PartitionKeys {
	return c.PartitionKeys
}
func (c ChangeBranchCountryCommand) GetProjector() AggregateProjector {
	return BranchProjector{}
}
