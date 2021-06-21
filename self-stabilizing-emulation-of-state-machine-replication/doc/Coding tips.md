
# Coding tips

Here are some high-level coding tips to keep the code fast. It's written for the snapshot project, but many parts can be relevant for other projects as well.

- Avoid busy-waiting. Condition variables or channels can be used instead.
- Avoid cloning objects if possible. The content of `RegisterArray` might be very large in the future if we want to evaluate the performance of coding when using large objects. Cloning large things unneccessarily is expensive.
    - One trick to avoid cloning `RegisterArray` when dealing with messages is to put a reference to the `RegisterArray` in a `Cow::Borrowed` in the message and then serialize the message. Serializing is essentially a cloning too, but one that has to be done anyway. Becasue of this an intermediate cloning can be avoided.
    - However, sometimes a clone of `RegisterArray` can't be avoided. One such example is `register_array_being_written`.
- Always acquire all locks in the same order everywhere to avoid deadlocks.
- Keep the locks locked as short time as possible to maximize the potential for concurrency.
    - Primitive types like `Int` can be copied.
    - Larger types like `RegisterArray` can be put into a message and serialized. After the serialization, the lock doesn't need to be acquired anymore.
- Sometimes the compiler/optimizer isn't as smart as I wish it could be. When you factor out common parts or clean the code in general, check the performance before and after by running `local_starter` with the arguments `-n 15 -w 15 -e -l 10 -o` to see if the performance is degraded or not.
    - An example: `RegisterArray` used to have a `Vector` struct (which was like `RegisterArray` but it stored general objects instead) instead of a `HashMap`. But switching to a `HashMap` directly increased the performance by 30%!
    - It's tempting to factor out the common parts of the 4 algorithms into a "BaseSnapshodeNode". But then one must be careful to check the performance of the already implemented algorithms.
- In addition to unit testing the code, I've found it useful to include assert statements where possible. One example is found in the client write operation. But remember that asserts should only be run when `debug_assertions` are set.