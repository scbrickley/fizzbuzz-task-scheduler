# FizzBuzz Task Scheduler

This project contains two binaries:

- `server`, which exposes an HTTP API that allows a user to schedule tasks
- `worker`, which pulls those scheduled tasks from a datastore and executes them at the specified time (or close enough to make little difference).

## Using the task scheduling service

To start the server, `cd` into the project root and run `cargo run --bin server`. The server will launch, and start listening for requests on port `8000`

The server exposes an API that sllows users to:

- Create new tasks
- List all tasks in the database, optionally filtering by task type and task completion status
- Look up a task by its ID
- Delete a task by its ID

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

`curl -X GET 'localhost:8000/tasks?filters.type=fizz&filters.status=scheduled'`

Valid arguments for the task filters are as follows:
- `filters.type` can be one of `fizz`, `buzz`, or `fizzbuzz` (capitalization does not matter in this case)
- `filters.status` can be either `scheduled` or `complete`

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

Worker processes are supposed to be able to run in parallel with one another, but due to the nature of the data store they are reading from, there are some practical limitations that prevent this from going smoothly 100% of the time.
