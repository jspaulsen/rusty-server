# Concerns
* Given how reads and writes are routed, there's a chance that with connects/disconnects locking the hashmap, it'll causes delays in routing messages to the process and clients.
* TODO: Consider using Papaya: https://docs.rs/papaya/latest/papaya/