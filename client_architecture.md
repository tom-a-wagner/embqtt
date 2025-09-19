Client Architecture:

The `Client` type has roughly the following architecture to let users perform requests
asynchronously and receive published messages on subscribed topics,
as well as automatically handle other communication with the broker such as keepalive.

    ┌────────────────────┐                                  ┌─────────────────────┐
    |   Async Requests   |                                  |   .published()      |
    │ .publish(),        │                                  | Async stream of inc.|
╔═══│ .subscribe() etc.  │═════════════ Client ═════════════| Publish packets     |═══╗
║   └──┬──────┬──────────┘                                  └─────────────────────┘   ║
║      │      │       ^                                                      ^        ║
║      |      |       ┆ Receive response via oneshot channel                 │        ║
║      |      |       ┆                                                      │        ║
║      |      | Register request packet id                 Publish Messages: │        ║
║      │      │ and response oneshot channel     Send to .published() stream │        ║
║      │      │       ┆                                                      │        ║
║      │      V       ┆                                                      │        ║
║      │   ┌────────────────────┐ Response to Req. (e.g. PubAck Message):    │        ║
║      │   │ List of            │ Send to resp. via oneshot channel       ┌────────────────────────┐
║      │   │ running requests   │<────────────────────────────────────────|         .run()         |
║      │   └────────────────────┘                                         │                        │
║      │                                                                  │ - Receive packets and  │
║      │                          Acknowledge Publish, do Keepalive, etc. |   handle based on type │
║      │ send out request       ┌─────────────────────────────────────────│ - Perform keepalive    │
║      │ directly               │                                     ┌──>│                        │
║      │                        │                                     │   └────────────────────────┘
║      V                        │                                     │               ║
║ ┌─────────────────────────┐   │                         ┌─────────────────────────┐ ║
║ │ Marshaller              │   │                         │ Demarshaller            │ ║
║ │ ┌─────────────────────┐ │   │                         │ ┌─────────────────────┐ │ ║
║ │ │ embedded_io_async:: │ │<──┘                         │ │ embedded_io_async:: │ │ ║
║ │ │     Write           │ │                             │ │     Read            │ │ ║
║ │ └─────────────────────┘ │                             │ └─────────────────────┘ │ ║
║ └─────────────────────────┘                             └─────────────────────────┘ ║
╚═════════════════════════════════════════════════════════════════════════════════════╝

All internal objects use asynchronous Mutexes, so that they can be shared between the .run() future
and any request futures.

The Marshaller and Demarshaller work with typed, non-serialized message types,
and will translate to the serialized MQTT protocol when reading/writing from the socket
to the broker.
