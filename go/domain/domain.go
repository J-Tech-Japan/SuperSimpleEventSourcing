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
				Country: ap.Country, // 他のフィールドは元の値をコピー
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
