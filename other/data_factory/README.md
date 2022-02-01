## Usage:
This is an example of constructing async state utilising `App::data_factory`

## Reason:
A `data_factory` would make sense in the following situations:
- When an async state does not necessarily have to be shared between workers/threads.

- When an async state would spawn tasks on `actix-rt`. If we centralized the state there it is possibile task distribution on workers/threads could become unbalanced.
(`actix-rt` would spawn tasks on local thread whenever it's called)

## Requirement:
- `rustc 1.58 stable`
- `redis` server listen on `127.0.0.1:6379`(or use `REDIS_URL` env argument when starting the example)

## Endpoints:
- Utilise a work load generator(e.g wrk) to benchmark the end points:

        http://127.0.0.1:8080/pool   prebuilt shared redis pool
        http://127.0.0.1:8080/pool2  data_factory redis pool

## Context:
The real world difference can vary depending on the work you are doing but in general it's a good idea to
spread *identical* async tasks evenly between threads and have as little cross thread synchronization as possible.
