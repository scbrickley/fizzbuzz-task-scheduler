# FizzBuzz Task Scheduler

This project contains two binaries:

- `server`, which exposes an HTTP API that allows a user to schedule tasks
- `worker`, which pulls those scheduled tasks from a datastore and executes them at the specified time (or close enough to make little difference).

## Using the task scheduling service

### Create the database

Run the following docker command:

```
docker run --rm --name fizzbuzz-task-db \
	-e POSTGRES_HOST_AUTH_METHOD=trust \
	-e POSTGRES_DB=fizzbuzz-task-db \
	-p 5432:5432 postgres:15
```

This will create a suitable PostgreSQL instance for the scheduler service to talk to. You can also run the `start-pg` bash script in the repo root to do the same thing.


Then run this `psql` command to create the table that the service will use:

```
psql --host=localhost -U postgres \
	-c 'CREATE TABLE tasks (
		id SERIAL PRIMARY KEY,
		tasktype VARCHAR NOT NULL,
		status VARCHAR NOT NULL,
		time TIMESTAMPTZ NOT NULL
	);'
```

You can also run the `init-db` bash script in the repo root.

_Note: the `sqlx` library that the scheduler uses to talk to postgres ostensibly has tools that handle table creation for you on start up. I could not get these to work as advertised, so I resorted to doing it the old fashioned way and storing the commands in the bash scripts mentioned above._

Once the postgres instance is running and has the appropriate tables, we can move on to compiling and running the server.

### Running the server

To start the server, `cd` into the project root and run `cargo run --bin server`. The server will launch, and start listening for requests on port `8000`

The server exposes an API that allows users to:

- Create new tasks
- List all tasks in the database, optionally filtering by task type and task completion status
- Look up a task by its ID
- Delete a task by its ID

It also exposes a few other endpoints that are only intended to be used by workers. These allow workers to:

- Check the timestamp of the next task to be executed
- Pull a task off the queue
- Mark a task as completed

### Creating new tasks

Once the server is running, you can run an HTTP POST request using curl to schedule a new task:

`curl -X POST -H 'content-type: application/json' -d '{"type": "Fizz", "time": "2023-03-28T11:59:59Z"}' localhost:8000/tasks`

Valid inputs for the JSON payload are as folows:

- `type` can either be `Fizz`, `Buzz`, or `FizzBuzz` (capitilization matters in this case)
- `time` can be any valid RFC3339 formatted timestamp. All times are interpreted as UTC.

A successful request should yield the task ID number.

### Listing all tasks

To view all known tasks, you can run an HTTP GET request like so:

`curl -X GET 'localhost:8000/tasks'`

This will yield a JSON payload detailing all tasks that the scheduler knows about. You can also add filters as query string parameters.

For example, to view all tasks of type `"Fizz"` that have not been executed yet, you can run the following curl command:

`curl -X GET 'localhost:8000/tasks?filters.type=fizz&filters.status=waiting'`

Valid arguments for the task filters are as follows:
- `filters.type` can be one of `fizz`, `buzz`, or `fizzbuzz` (capitalization does not matter in this case)
- `filters.status` can be either `waiting`, `claimed`, or `complete`

Both filters are optional. You can run your request with zero, one, or both filters set.

### Getting a task by its ID

To lookup a specific task by its task ID number, run the following `curl` command:
`curl -X GET localhost:8000/tasks/<id>`

Any positive integer is a valid ID number.

This endpoint will return the JSON data for the task.

### Deleting a task by its ID

To delete a task, you can run a similar `curl` command using the DELETE method:

`curl -X DELETE localhost:8000/tasks/<id>`

This endpoint will return the JSON data for the deleted task.

## Running the worker binary

To run a worker, `cd` into the project root and run the following command:

`cargo run --bin worker`

This will run a worker process that will periodically check for any scheduled tasks that need to be run.

- `Fizz` tasks will cause the worker to sleep for 3 seconds, starting at the scheduled execution time, before printing out the word `Fizz`, followed by the task ID number.
- `Buzz` tasks will cause the worker to sleep for 5 seconds, starting at the scheduled execution time, before printing out the word `Buzz`, followed by the task ID number.
- `FizzBuzz` tasks will run immediately once the scheduled execution time is reached or passed, before printing out the word `FizzBuzz`, followed by the current time.

Worker processes can run in parallel with one another, each pulling tasksk off the queue and running them at the specified execution time.

Each worker follows these steps:

1. Check the timestamp for the task scheduled to run next. If there are no tasks currently on the queue, defer (i.e., sleep for 1 second, and go back to step 1)
2. If it's not yet time to run the task, defer. Otheriwse, tell the server to return the task info and mark the task as claimed. 
3. Run the task according to the task type.
4. Once the task is done running, tell the server to mark the task as complete.
