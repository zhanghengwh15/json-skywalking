## ADDED Requirements

### Requirement: Create task branch group record
The system SHALL allow creating a task-branch-group association record with tb_name, task_id, branch_name, and group_type.

#### Scenario: Successful creation via Tauri command
- **WHEN** the frontend invokes `task_branch_group_create` with valid parameters
- **THEN** the system inserts a new record and returns the generated id

#### Scenario: Successful creation via HTTP API
- **WHEN** a POST request is sent to `/api/task-branch-groups` with valid JSON body
- **THEN** the system inserts a new record and returns the created object with id

#### Scenario: Successful creation via CLI
- **WHEN** user runs `dev-tools.exe task-branch-group create --tb-name <n> --task-id <id> --branch-name <b> --group-type <1|2>`
- **THEN** the system inserts a new record and prints the created record as JSON

### Requirement: Read task branch group records
The system SHALL support querying task branch group records by id or listing all records with optional task_id filter.

#### Scenario: List all records
- **WHEN** the frontend invokes `task_branch_group_list` or GET `/api/task-branch-groups`
- **THEN** the system returns an array of all records with rec_status = 1

#### Scenario: Filter by task_id
- **WHEN** the frontend invokes `task_branch_group_list` with task_id parameter or GET `/api/task-branch-groups?task_id=<id>`
- **THEN** the system returns only records matching the given task_id

#### Scenario: Get single record by id
- **WHEN** the frontend invokes `task_branch_group_get` with id or GET `/api/task-branch-groups/<id>`
- **THEN** the system returns the matching record or a not-found error

#### Scenario: CLI list with filter
- **WHEN** user runs `dev-tools.exe task-branch-group list --task-id <id>`
- **THEN** the system prints matching records as JSON array

### Requirement: Update task branch group record
The system SHALL allow updating tb_name, branch_name, group_type, and rec_status of an existing record.

#### Scenario: Successful update via Tauri command
- **WHEN** the frontend invokes `task_branch_group_update` with id and updated fields
- **THEN** the system updates the record, refreshes modify_time, and returns the updated record

#### Scenario: Successful update via HTTP API
- **WHEN** a PUT request is sent to `/api/task-branch-groups/<id>` with updated fields
- **THEN** the system updates the record and returns the updated object

#### Scenario: Update via CLI
- **WHEN** user runs `dev-tools.exe task-branch-group update <id> --branch-name <new>`
- **THEN** the system updates the specified fields and prints the updated record

### Requirement: Delete task branch group record
The system SHALL support soft-deleting a record by setting rec_status to 0.

#### Scenario: Soft delete via Tauri command
- **WHEN** the frontend invokes `task_branch_group_delete` with id
- **THEN** the system sets rec_status to 0 for the record

#### Scenario: Soft delete via HTTP API
- **WHEN** a DELETE request is sent to `/api/task-branch-groups/<id>`
- **THEN** the system sets rec_status to 0 and returns success

#### Scenario: Delete via CLI
- **WHEN** user runs `dev-tools.exe task-branch-group delete <id>`
- **THEN** the system performs soft delete and prints confirmation

### Requirement: Database schema initialization
The system SHALL create the task_branch_group table and modify_time trigger on database open if they do not exist.

#### Scenario: First launch after upgrade
- **WHEN** the application starts and opens data.db
- **THEN** the system ensures task_branch_group table and trigger exist without error
