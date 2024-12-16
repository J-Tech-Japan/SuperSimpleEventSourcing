// type EventPayloadBase = {
//     IsEventPayload: () => boolean;
// };
// type EventPayload<T> = T & EventPayloadBase;
interface EventPayload {
    IsEventPayload() : boolean;
}