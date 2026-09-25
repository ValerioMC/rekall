-- H2 2.4.240; 
SET DB_CLOSE_DELAY -1;         
;              
CREATE USER IF NOT EXISTS "REKALL" SALT '43165c684ec6b655' HASH '3172a612fc2b4979f790bdad83d23d4c70ad224819e9c56f1d0f72615878af9c' ADMIN;      
CREATE CACHED TABLE "PUBLIC"."DATABASECHANGELOG"(
    "ID" CHARACTER VARYING(255) NOT NULL,
    "AUTHOR" CHARACTER VARYING(255) NOT NULL,
    "FILENAME" CHARACTER VARYING(255) NOT NULL,
    "DATEEXECUTED" TIMESTAMP NOT NULL,
    "ORDEREXECUTED" INTEGER NOT NULL,
    "EXECTYPE" CHARACTER VARYING(10) NOT NULL,
    "MD5SUM" CHARACTER VARYING(35),
    "DESCRIPTION" CHARACTER VARYING(255),
    "COMMENTS" CHARACTER VARYING(255),
    "TAG" CHARACTER VARYING(255),
    "LIQUIBASE" CHARACTER VARYING(20),
    "CONTEXTS" CHARACTER VARYING(255),
    "LABELS" CHARACTER VARYING(255),
    "DEPLOYMENT_ID" CHARACTER VARYING(10)
);   
-- 37 +/- SELECT COUNT(*) FROM PUBLIC.DATABASECHANGELOG;       
INSERT INTO "PUBLIC"."DATABASECHANGELOG" VALUES
('001-environment', 'rekall', 'db/changelog/001-core.yaml', TIMESTAMP '2026-09-25 08:32:31.75617', 1, 'EXECUTED', '9:f04297b24b783666ca854eaa9ddbc8c2', 'createTable tableName=environment; addUniqueConstraint constraintName=uq_environment_label, tableName=environment', '', NULL, '5.0.3', NULL, NULL, '0325150251'),
('001-project', 'rekall', 'db/changelog/001-core.yaml', TIMESTAMP '2026-09-25 08:32:31.761976', 2, 'EXECUTED', '9:5febbbe49b4ec44cdd86568ad41f78a2', 'createTable tableName=project; addUniqueConstraint constraintName=uq_project_name, tableName=project', '', NULL, '5.0.3', NULL, NULL, '0325150251'),
('001-task', 'rekall', 'db/changelog/001-core.yaml', TIMESTAMP '2026-09-25 08:32:31.776086', 3, 'EXECUTED', '9:cc9536491d5cd9c30bcd2ef933dfd3a8', 'createTable tableName=task; addForeignKeyConstraint baseTableName=task, constraintName=fk_task_project, referencedTableName=project; addForeignKeyConstraint baseTableName=task, constraintName=fk_task_environment, referencedTableName=environment; a...', '', NULL, '5.0.3', NULL, NULL, '0325150251'),
('001-document', 'rekall', 'db/changelog/001-core.yaml', TIMESTAMP '2026-09-25 08:32:31.800761', 4, 'EXECUTED', '9:335454fd015e30726f4f2cfed2470422', 'createTable tableName=document; addForeignKeyConstraint baseTableName=document, constraintName=fk_document_project, referencedTableName=project; addForeignKeyConstraint baseTableName=document, constraintName=fk_document_task, referencedTableName=t...', '', NULL, '5.0.3', NULL, NULL, '0325150251'),
('002-document-task', 'rekall', 'db/changelog/002-documents-on-many-tasks.yaml', TIMESTAMP '2026-09-25 08:32:31.811091', 5, 'EXECUTED', '9:21501eddf627ecece80e51416dda1120', 'createTable tableName=document_task; addPrimaryKey constraintName=pk_document_task, tableName=document_task; addForeignKeyConstraint baseTableName=document_task, constraintName=fk_document_task_document, referencedTableName=document; addForeignKey...', '', NULL, '5.0.3', NULL, NULL, '0325150251'),
('002-carry-task-notes', 'rekall', 'db/changelog/002-documents-on-many-tasks.yaml', TIMESTAMP '2026-09-25 08:32:31.814239', 6, 'EXECUTED', '9:f0231b557056ce68438360155acd3bdc', 'sql', '', NULL, '5.0.3', NULL, NULL, '0325150251'),
('002-drop-ownerless-notes', 'rekall', 'db/changelog/002-documents-on-many-tasks.yaml', TIMESTAMP '2026-09-25 08:32:31.816659', 7, 'EXECUTED', '9:187e26aa4170054b407dfd4da63d57a0', 'delete tableName=document', '', NULL, '5.0.3', NULL, NULL, '0325150251'),
('002-document-loses-its-owners', 'rekall', 'db/changelog/002-documents-on-many-tasks.yaml', TIMESTAMP '2026-09-25 08:32:31.841523', 8, 'EXECUTED', '9:21a78ff93316e16d0d60031d9b20ccfc', 'dropForeignKeyConstraint baseTableName=document, constraintName=fk_document_project; dropForeignKeyConstraint baseTableName=document, constraintName=fk_document_task; dropForeignKeyConstraint baseTableName=document, constraintName=fk_document_envi...', '', NULL, '5.0.3', NULL, NULL, '0325150251'),
('002-drop-environment', 'rekall', 'db/changelog/002-documents-on-many-tasks.yaml', TIMESTAMP '2026-09-25 08:32:31.851163', 9, 'EXECUTED', '9:717d5bfe2713ecb05989a28a21982646', 'dropForeignKeyConstraint baseTableName=task, constraintName=fk_task_environment; dropColumn columnName=environment_id, tableName=task; dropTable tableName=environment', '', NULL, '5.0.3', NULL, NULL, '0325150251'),
('003-company', 'rekall', 'db/changelog/003-company.yaml', TIMESTAMP '2026-09-25 08:32:31.857658', 10, 'EXECUTED', '9:b1b07b089188eb4fb767af89baa04604', 'createTable tableName=company; addUniqueConstraint constraintName=uq_company_name, tableName=company', '', NULL, '5.0.3', NULL, NULL, '0325150251'),
('003-holding-company-for-existing-projects', 'rekall', 'db/changelog/003-company.yaml', TIMESTAMP '2026-09-25 08:32:31.863239', 11, 'MARK_RAN', '9:5565eea334b7900dcb7b02373537bdc8', 'sql', '', NULL, '5.0.3', NULL, NULL, '0325150251'),
('003-project-belongs-to-a-company', 'rekall', 'db/changelog/003-company.yaml', TIMESTAMP '2026-09-25 08:32:31.881243', 12, 'EXECUTED', '9:cea6f2d701fb43f07e8b64f1fbf0ae47', 'addColumn tableName=project; sql; addNotNullConstraint columnName=company_id, tableName=project; addForeignKeyConstraint baseTableName=project, constraintName=fk_project_company, referencedTableName=company', '', NULL, '5.0.3', NULL, NULL, '0325150251');  
INSERT INTO "PUBLIC"."DATABASECHANGELOG" VALUES
('003-project-name-unique-per-company', 'rekall', 'db/changelog/003-company.yaml', TIMESTAMP '2026-09-25 08:32:31.887016', 13, 'EXECUTED', '9:695a2e58d1ee9774f6a3345bd5bc9273', 'dropUniqueConstraint constraintName=uq_project_name, tableName=project; addUniqueConstraint constraintName=uq_project_company_name, tableName=project', '', NULL, '5.0.3', NULL, NULL, '0325150251'),
('004-project-label-and-title', 'rekall', 'db/changelog/004-label-title-description.yaml', TIMESTAMP '2026-09-25 08:32:31.90325', 14, 'EXECUTED', '9:245e9062c527e7a958fdecb5224df8d6', 'addColumn tableName=project; sql; addNotNullConstraint columnName=title, tableName=project; renameColumn newColumnName=label, oldColumnName=name, tableName=project; sql; dropUniqueConstraint constraintName=uq_project_company_name, tableName=projec...', '', NULL, '5.0.3', NULL, NULL, '0325150251'),
('004-task-label-and-title', 'rekall', 'db/changelog/004-label-title-description.yaml', TIMESTAMP '2026-09-25 08:32:31.917253', 15, 'EXECUTED', '9:eec7b50a085ec73ec734499b91eb0ad3', 'addColumn tableName=task; sql; addNotNullConstraint columnName=title, tableName=task; renameColumn newColumnName=label, oldColumnName=name, tableName=task; sql; dropUniqueConstraint constraintName=uq_task_project_name, tableName=task; addUniqueCon...', '', NULL, '5.0.3', NULL, NULL, '0325150251'),
('005-wrapup', 'rekall', 'db/changelog/005-wrapup.yaml', TIMESTAMP '2026-09-25 08:32:31.925204', 16, 'EXECUTED', '9:9444a6bedc771ceca3141d1c48d2c6ab', 'createTable tableName=wrapup; addUniqueConstraint constraintName=uq_wrapup_task, tableName=wrapup; addForeignKeyConstraint baseTableName=wrapup, constraintName=fk_wrapup_task, referencedTableName=task', '', NULL, '5.0.3', NULL, NULL, '0325150251'),
('006-time-entry', 'rekall', 'db/changelog/006-time-entry.yaml', TIMESTAMP '2026-09-25 08:32:31.932141', 17, 'EXECUTED', '9:8d619aaf8e56e3922925a1fe75cb6742', 'createTable tableName=time_entry; addForeignKeyConstraint baseTableName=time_entry, constraintName=fk_time_entry_task, referencedTableName=task', '', NULL, '5.0.3', NULL, NULL, '0325150251'),
('007-project-blueprint', 'rekall', 'db/changelog/007-project-blueprint.yaml', TIMESTAMP '2026-09-25 08:32:31.938996', 18, 'EXECUTED', '9:11dccff747284a50f8f7aba07e17facd', 'addColumn tableName=project', '', NULL, '5.0.3', NULL, NULL, '0325150251'),
('008-project-repo-folder', 'rekall', 'db/changelog/008-project-repo-folder.yaml', TIMESTAMP '2026-09-25 08:32:31.944888', 19, 'EXECUTED', '9:9e445e8e6fd6ff43201e1ebea735cc3b', 'addColumn tableName=project', '', NULL, '5.0.3', NULL, NULL, '0325150251'),
('009-task-step', 'rekall', 'db/changelog/009-task-step.yaml', TIMESTAMP '2026-09-25 08:32:31.951432', 20, 'EXECUTED', '9:8d0b927c8c4eb8090792d84369a28fa2', 'createTable tableName=task_step; addForeignKeyConstraint baseTableName=task_step, constraintName=fk_task_step_task, referencedTableName=task; createIndex indexName=idx_task_step_task, tableName=task_step', '', NULL, '5.0.3', NULL, NULL, '0325150251'),
('010-task-step-state', 'rekall', 'db/changelog/010-task-step-state.yaml', TIMESTAMP '2026-09-25 08:32:31.967969', 21, 'EXECUTED', '9:d12588bbeebba66fe945865cb5cd0508', 'addColumn tableName=task_step; update tableName=task_step; dropColumn columnName=done, tableName=task_step', '', NULL, '5.0.3', NULL, NULL, '0325150251'),
('011-claude-session', 'rekall', 'db/changelog/011-claude-session.yaml', TIMESTAMP '2026-09-25 08:32:31.989209', 22, 'EXECUTED', '9:0f90e7a625c3e69a84aa93b273917b43', 'createTable tableName=claude_session; addForeignKeyConstraint baseTableName=claude_session, constraintName=fk_claude_session_task, referencedTableName=task; createIndex indexName=idx_claude_session_task, tableName=claude_session; createIndex index...', '', NULL, '5.0.3', NULL, NULL, '0325150251'),
('012-claude-session-model', 'rekall', 'db/changelog/012-claude-session-model.yaml', TIMESTAMP '2026-09-25 08:32:31.999203', 23, 'EXECUTED', '9:d6be0ce9d5747b8ef3951fa0e5a7dc92', 'addColumn tableName=claude_session', '', NULL, '5.0.3', NULL, NULL, '0325150251');
INSERT INTO "PUBLIC"."DATABASECHANGELOG" VALUES
('013-claude-session-effort', 'rekall', 'db/changelog/013-claude-session-effort.yaml', TIMESTAMP '2026-09-25 08:32:32.007969', 24, 'EXECUTED', '9:6dc1fa0df17163f9efac248c61ddbb81', 'addColumn tableName=claude_session', '', NULL, '5.0.3', NULL, NULL, '0325150251'),
('014-task-review-state', 'rekall', 'db/changelog/014-task-review-state.yaml', TIMESTAMP '2026-09-25 08:32:32.026269', 25, 'EXECUTED', '9:a8f973e52106b5a3e7389b6d4a4879ba', 'addColumn tableName=task', '', NULL, '5.0.3', NULL, NULL, '0325150251'),
('015-task-wrapup-directive', 'rekall', 'db/changelog/015-task-wrapup-directive.yaml', TIMESTAMP '2026-09-25 08:32:32.036721', 26, 'EXECUTED', '9:f97e11756666f18d81e4765e1baaaaa9', 'addColumn tableName=task', '', NULL, '5.0.3', NULL, NULL, '0325150251'),
('016-task-step-draft', 'rekall', 'db/changelog/016-task-step-draft.yaml', TIMESTAMP '2026-09-25 08:32:32.041036', 27, 'EXECUTED', '9:0909134dfa027dd5e9860f7d528c449f', 'dropDefaultValue columnName=state, tableName=task_step; addDefaultValue columnName=state, tableName=task_step', '', NULL, '5.0.3', NULL, NULL, '0325150251'),
('017-drop-claude-session', 'rekall', 'db/changelog/017-drop-claude-session.yaml', TIMESTAMP '2026-09-25 08:32:32.044865', 28, 'EXECUTED', '9:4a9a9a94ff52b30b7c8ff550aef9ee82', 'dropTable tableName=claude_message; dropTable tableName=claude_session', '', NULL, '5.0.3', NULL, NULL, '0325150251'),
('018-drop-task-wrapup-directive', 'rekall', 'db/changelog/018-drop-task-wrapup-directive.yaml', TIMESTAMP '2026-09-25 08:32:32.054883', 29, 'EXECUTED', '9:48dad9a8c3a0b622e58acb7bfe826c52', 'dropColumn tableName=task', '', NULL, '5.0.3', NULL, NULL, '0325150251'),
('019-commit-reference', 'rekall', 'db/changelog/019-commit-reference.yaml', TIMESTAMP '2026-09-25 08:32:32.062784', 30, 'EXECUTED', '9:0ef5ebfb4d8b3cdbccecbed3e98e0e2f', 'createTable tableName=commit_reference; addForeignKeyConstraint baseTableName=commit_reference, constraintName=fk_commit_reference_task, referencedTableName=task; addForeignKeyConstraint baseTableName=commit_reference, constraintName=fk_commit_ref...', '', NULL, '5.0.3', NULL, NULL, '0325150251'),
('020-commit-reference-diff', 'rekall', 'db/changelog/020-commit-reference-diff.yaml', TIMESTAMP '2026-09-25 08:32:32.071456', 31, 'EXECUTED', '9:e215000250eb6edcae8155cfe94f5c08', 'addColumn tableName=commit_reference', '', NULL, '5.0.3', NULL, NULL, '0325150251'),
('021-commit-reference-in-context', 'rekall', 'db/changelog/021-commit-reference-in-context.yaml', TIMESTAMP '2026-09-25 08:32:32.075912', 32, 'EXECUTED', '9:9a40b44ede9b22d4041d9738e8ccd8e6', 'addColumn tableName=commit_reference', '', NULL, '5.0.3', NULL, NULL, '0325150251'),
('022-project-auto-commit', 'rekall', 'db/changelog/022-project-auto-commit.yaml', TIMESTAMP '2026-09-25 08:32:32.080732', 33, 'EXECUTED', '9:cde2a9898240a9208c5fa92ea5166cf2', 'addColumn tableName=project', '', NULL, '5.0.3', NULL, NULL, '0325150251'),
('023-tag', 'rekall', 'db/changelog/023-tag.yaml', TIMESTAMP '2026-09-25 08:32:32.089341', 34, 'EXECUTED', '9:cbea08a3e65eda0d7dd396f4ebd0e422', 'createTable tableName=tag; addColumn tableName=task; addForeignKeyConstraint baseTableName=task, constraintName=fk_task_tag, referencedTableName=tag', '', NULL, '5.0.3', NULL, NULL, '0325150251'),
('024-task-revision', 'rekall', 'db/changelog/024-task-revision.yaml', TIMESTAMP '2026-09-25 08:32:32.096122', 35, 'EXECUTED', '9:52abf48777ca5cb75ba4762d0a8f9b5c', 'createTable tableName=task_revision; addForeignKeyConstraint baseTableName=task_revision, constraintName=fk_task_revision_task, referencedTableName=task; createIndex indexName=ix_task_revision_task_kind, tableName=task_revision', '', NULL, '5.0.3', NULL, NULL, '0325150251'),
('025-document-context-mode', 'rekall', 'db/changelog/025-document-context-mode.yaml', TIMESTAMP '2026-09-25 08:32:32.101316', 36, 'EXECUTED', '9:ac0eeecdc236ec31a56e91faa848afc2', 'addColumn tableName=document', '', NULL, '5.0.3', NULL, NULL, '0325150251'),
('026-run-queue', 'rekall', 'db/changelog/026-run-queue.yaml', TIMESTAMP '2026-09-25 08:32:32.112436', 37, 'EXECUTED', '9:898a42110e03e873ac9e73fa7b219b5b', 'createTable tableName=run_queue; createTable tableName=run_queue_item; addForeignKeyConstraint baseTableName=run_queue_item, constraintName=fk_run_queue_item_task, referencedTableName=task; createIndex indexName=ix_run_queue_item_position, tableNa...', '', NULL, '5.0.3', NULL, NULL, '0325150251');  
CREATE CACHED TABLE "PUBLIC"."DATABASECHANGELOGLOCK"(
    "ID" INTEGER NOT NULL,
    "LOCKED" BOOLEAN NOT NULL,
    "LOCKGRANTED" TIMESTAMP,
    "LOCKEDBY" CHARACTER VARYING(255)
);          
ALTER TABLE "PUBLIC"."DATABASECHANGELOGLOCK" ADD CONSTRAINT "PUBLIC"."PK_DATABASECHANGELOGLOCK" PRIMARY KEY("ID");             
-- 1 +/- SELECT COUNT(*) FROM PUBLIC.DATABASECHANGELOGLOCK;    
INSERT INTO "PUBLIC"."DATABASECHANGELOGLOCK" VALUES
(1, FALSE, NULL, NULL);    
CREATE CACHED TABLE "PUBLIC"."COMPANY"(
    "ID" UUID NOT NULL,
    "NAME" CHARACTER VARYING(120) NOT NULL,
    "DESCRIPTION" CHARACTER VARYING(100000),
    "CREATED_AT" TIMESTAMP NOT NULL,
    "UPDATED_AT" TIMESTAMP NOT NULL
);           
ALTER TABLE "PUBLIC"."COMPANY" ADD CONSTRAINT "PUBLIC"."PK_COMPANY" PRIMARY KEY("ID");         
-- 6 +/- SELECT COUNT(*) FROM PUBLIC.COMPANY;  
INSERT INTO "PUBLIC"."COMPANY" VALUES
(UUID 'f360ff3a-135d-458e-9e4f-6e5c6ae02000', 'Globex Corp', 'x', TIMESTAMP '2026-09-25 08:32:37.8184', TIMESTAMP '2026-09-25 08:32:37.847217'),
(UUID '80ba5f6b-b765-4a47-8487-14d2356de37e', 'Acme', 'Il cliente', TIMESTAMP '2026-09-25 08:35:28.548143', TIMESTAMP '2026-09-25 08:35:28.548184'),
(UUID 'ab5c14cd-bd03-40a5-bcfd-67279e4f89a4', 'Globex', NULL, TIMESTAMP '2026-09-25 08:35:28.642503', TIMESTAMP '2026-09-25 08:35:28.642518'),
(UUID '4c231b5f-44bd-4468-8089-59a982ab5243', 'Co fd0d3d', NULL, TIMESTAMP '2026-09-25 08:35:30.764391', TIMESTAMP '2026-09-25 08:35:30.764409'),
(UUID '255cfb53-c59f-43eb-a11b-ea632f186067', 'Co ed0e6f', NULL, TIMESTAMP '2026-09-25 08:35:30.943471', TIMESTAMP '2026-09-25 08:35:30.943482'),
(UUID '6174fb70-074b-4331-aa61-7fd215d86275', 'Co 4e99be', NULL, TIMESTAMP '2026-09-25 08:35:31.088813', TIMESTAMP '2026-09-25 08:35:31.088822');               
CREATE CACHED TABLE "PUBLIC"."TAG"(
    "ID" UUID NOT NULL,
    "NAME" CHARACTER VARYING(60) NOT NULL,
    "ICON" CHARACTER VARYING(40) NOT NULL,
    "COLOR" CHARACTER VARYING(40) NOT NULL,
    "CREATED_AT" TIMESTAMP NOT NULL,
    "UPDATED_AT" TIMESTAMP NOT NULL
);      
ALTER TABLE "PUBLIC"."TAG" ADD CONSTRAINT "PUBLIC"."PK_TAG" PRIMARY KEY("ID"); 
-- 0 +/- SELECT COUNT(*) FROM PUBLIC.TAG;      
CREATE CACHED TABLE "PUBLIC"."TASK_REVISION"(
    "ID" UUID NOT NULL,
    "TASK_ID" UUID NOT NULL,
    "KIND" CHARACTER VARYING(20) NOT NULL,
    "BODY_MARKDOWN" CHARACTER VARYING(100000) NOT NULL,
    "WRITTEN_BY" CHARACTER VARYING(20),
    "WRITTEN_AT" TIMESTAMP,
    "CREATED_AT" TIMESTAMP NOT NULL
);               
ALTER TABLE "PUBLIC"."TASK_REVISION" ADD CONSTRAINT "PUBLIC"."PK_TASK_REVISION" PRIMARY KEY("ID");             
-- 2 +/- SELECT COUNT(*) FROM PUBLIC.TASK_REVISION;            
INSERT INTO "PUBLIC"."TASK_REVISION" VALUES
(UUID 'a5c2bcaa-c7cc-40d3-9622-0354548b30ae', UUID '7aee22c5-55d1-47bf-aafa-102f57870d14', 'WRAPUP', 'Scritto a mano.', 'HAND', TIMESTAMP '2026-09-25 08:35:29.323209', TIMESTAMP '2026-09-25 08:35:29.665726'),
(UUID '9b6dadb1-7851-416c-8f7f-3fcd6c41aeac', UUID '7aee22c5-55d1-47bf-aafa-102f57870d14', 'WRAPUP', 'Riscritto.', 'CLAUDE', TIMESTAMP '2026-09-25 08:35:29.671379', TIMESTAMP '2026-09-25 08:35:30.168711');     
CREATE INDEX "PUBLIC"."IX_TASK_REVISION_TASK_KIND" ON "PUBLIC"."TASK_REVISION"("TASK_ID" NULLS FIRST, "KIND" NULLS FIRST, "CREATED_AT" NULLS FIRST);           
CREATE CACHED TABLE "PUBLIC"."WRAPUP"(
    "ID" UUID NOT NULL,
    "TASK_ID" UUID NOT NULL,
    "BODY_MARKDOWN" CHARACTER VARYING(20000) NOT NULL,
    "WRITTEN_BY" CHARACTER VARYING(20) NOT NULL,
    "CREATED_AT" TIMESTAMP NOT NULL,
    "UPDATED_AT" TIMESTAMP NOT NULL
);
ALTER TABLE "PUBLIC"."WRAPUP" ADD CONSTRAINT "PUBLIC"."PK_WRAPUP" PRIMARY KEY("ID");           
-- 2 +/- SELECT COUNT(*) FROM PUBLIC.WRAPUP;   
INSERT INTO "PUBLIC"."WRAPUP" VALUES
(UUID 'fcfd51a6-6b0a-43bd-b9c2-29b283b1f2a4', UUID '7aee22c5-55d1-47bf-aafa-102f57870d14', 'Scritto a mano.', 'HAND', TIMESTAMP '2026-09-25 08:35:29.323185', TIMESTAMP '2026-09-25 08:35:30.173312'),
(UUID 'b7b4366c-945e-490d-a992-7b22cfc8db1c', UUID '2550ba76-9e87-4d75-8ab8-9ea96949d328', U&'## Stato\000a\000aLe righe sono aggregate.', 'CLAUDE', TIMESTAMP '2026-09-25 08:35:29.642316', TIMESTAMP '2026-09-25 08:35:29.642325');              
CREATE CACHED TABLE "PUBLIC"."RUN_QUEUE"(
    "ID" UUID NOT NULL,
    "STATE" CHARACTER VARYING(20) NOT NULL,
    "START_AT" TIMESTAMP,
    "CEILING_PERCENT" INTEGER,
    "SKIP_PERMISSIONS" BOOLEAN DEFAULT FALSE NOT NULL,
    "MODEL" CHARACTER VARYING(20),
    "EFFORT" CHARACTER VARYING(20),
    "HOLD_UNTIL" TIMESTAMP,
    "HOLD_REASON" CHARACTER VARYING(500),
    "CREATED_AT" TIMESTAMP NOT NULL,
    "UPDATED_AT" TIMESTAMP NOT NULL
);         
ALTER TABLE "PUBLIC"."RUN_QUEUE" ADD CONSTRAINT "PUBLIC"."PK_RUN_QUEUE" PRIMARY KEY("ID");     
-- 1 +/- SELECT COUNT(*) FROM PUBLIC.RUN_QUEUE;
INSERT INTO "PUBLIC"."RUN_QUEUE" VALUES
(UUID '9c963ff4-a3f1-44d6-bf1d-dd3ed598711a', 'IDLE', NULL, 80, TRUE, 'sonnet', 'high', NULL, NULL, TIMESTAMP '2026-09-25 08:32:39.581017', TIMESTAMP '2026-09-25 08:35:30.424977');   
CREATE CACHED TABLE "PUBLIC"."TASK"(
    "ID" UUID NOT NULL,
    "LABEL" CHARACTER VARYING(160) NOT NULL,
    "STATUS" CHARACTER VARYING(20) NOT NULL,
    "DESCRIPTION" CHARACTER VARYING(100000),
    "PROJECT_ID" UUID NOT NULL,
    "CREATED_AT" TIMESTAMP NOT NULL,
    "UPDATED_AT" TIMESTAMP NOT NULL,
    "TITLE" CHARACTER VARYING(200) NOT NULL,
    "REVIEW_STATE" CHARACTER VARYING(20) DEFAULT 'OPEN' NOT NULL,
    "CLAIMED_AT" TIMESTAMP,
    "ACCEPTED_AT" TIMESTAMP,
    "REVIEW_NOTE" CHARACTER VARYING(2000),
    "TAG_ID" UUID
);          
ALTER TABLE "PUBLIC"."TASK" ADD CONSTRAINT "PUBLIC"."PK_TASK" PRIMARY KEY("ID");               
-- 21 +/- SELECT COUNT(*) FROM PUBLIC.TASK;    
INSERT INTO "PUBLIC"."TASK" VALUES
(UUID '2550ba76-9e87-4d75-8ab8-9ea96949d328', 'report-builder', 'IN_PROGRESS', U&'## Goal\000a\000aBuild it.', UUID 'bfc1be35-3de7-4cd7-ad87-ce6ec42674f5', TIMESTAMP '2026-09-25 08:35:28.899163', TIMESTAMP '2026-09-25 08:35:28.899176', 'Report builder', 'OPEN', NULL, NULL, NULL, NULL),
(UUID '7aee22c5-55d1-47bf-aafa-102f57870d14', 'retry-policy', 'IN_PROGRESS', 'desc', UUID 'bfc1be35-3de7-4cd7-ad87-ce6ec42674f5', TIMESTAMP '2026-09-25 08:35:28.914001', TIMESTAMP '2026-09-25 08:35:29.676025', 'Retry policy v2', 'CLAIMED', TIMESTAMP '2026-09-25 08:35:29.674248', NULL, NULL, NULL),
(UUID 'cd11f174-4b24-4649-b74c-4b295d818090', 'setup', 'BLOCKED', NULL, UUID '58ddefe2-ed26-4366-94b0-ff7154dd1746', TIMESTAMP '2026-09-25 08:35:28.925136', TIMESTAMP '2026-09-25 08:35:29.856982', 'Setup', 'DONE', TIMESTAMP '2026-09-25 08:35:29.854276', TIMESTAMP '2026-09-25 08:35:29.854276', NULL, NULL),
(UUID '24865959-53df-4432-b815-e6d4ba549165', 't0', 'TODO', 'Descr 0', UUID '88ba951a-8d7b-4901-a68b-8e90c0e85fa6', TIMESTAMP '2026-09-25 08:35:30.79128', TIMESTAMP '2026-09-25 08:35:30.791292', U&'Task 0 \00b7 \2713', 'OPEN', NULL, NULL, NULL, NULL),
(UUID 'dcc6e78e-ab4e-4948-8b39-c021fe9a4f43', 't1', 'TODO', 'Descr 1', UUID '88ba951a-8d7b-4901-a68b-8e90c0e85fa6', TIMESTAMP '2026-09-25 08:35:30.801937', TIMESTAMP '2026-09-25 08:35:30.801945', U&'Task 1 \00b7 \2713', 'OPEN', NULL, NULL, NULL, NULL),
(UUID '826123bd-a656-4eb5-8051-bef2dc7a8685', 't2', 'TODO', 'Descr 2', UUID '88ba951a-8d7b-4901-a68b-8e90c0e85fa6', TIMESTAMP '2026-09-25 08:35:30.809489', TIMESTAMP '2026-09-25 08:35:30.809497', U&'Task 2 \00b7 \2713', 'OPEN', NULL, NULL, NULL, NULL),
(UUID '770a6413-73fc-4004-b61d-843c2c4502ee', 't3', 'TODO', 'Descr 3', UUID '88ba951a-8d7b-4901-a68b-8e90c0e85fa6', TIMESTAMP '2026-09-25 08:35:30.819707', TIMESTAMP '2026-09-25 08:35:30.819717', U&'Task 3 \00b7 \2713', 'OPEN', NULL, NULL, NULL, NULL),
(UUID '625ddb04-04f1-41e6-8326-3f22ff4e2ca3', 't4', 'TODO', 'Descr 4', UUID '88ba951a-8d7b-4901-a68b-8e90c0e85fa6', TIMESTAMP '2026-09-25 08:35:30.829054', TIMESTAMP '2026-09-25 08:35:30.829063', U&'Task 4 \00b7 \2713', 'OPEN', NULL, NULL, NULL, NULL),
(UUID 'd0b43f69-6289-4e50-935c-f9af304cec87', 't5', 'TODO', 'Descr 5', UUID '88ba951a-8d7b-4901-a68b-8e90c0e85fa6', TIMESTAMP '2026-09-25 08:35:30.839076', TIMESTAMP '2026-09-25 08:35:30.839084', U&'Task 5 \00b7 \2713', 'OPEN', NULL, NULL, NULL, NULL),
(UUID '43fff879-efaa-4b87-bbfd-8910ebaad234', 't0', 'TODO', 'Descr 0', UUID '47b41d88-a01f-4322-9eee-f2d304e40a11', TIMESTAMP '2026-09-25 08:35:30.958109', TIMESTAMP '2026-09-25 08:35:30.958118', U&'Task 0 \00b7 \2713', 'OPEN', NULL, NULL, NULL, NULL),
(UUID '6e3a5ee1-bda8-4dbc-b205-1f288b89bd92', 't1', 'TODO', 'Descr 1', UUID '47b41d88-a01f-4322-9eee-f2d304e40a11', TIMESTAMP '2026-09-25 08:35:30.967065', TIMESTAMP '2026-09-25 08:35:30.967074', U&'Task 1 \00b7 \2713', 'OPEN', NULL, NULL, NULL, NULL),
(UUID '70786d31-eda4-4e63-accc-ad19868291ae', 't2', 'TODO', 'Descr 2', UUID '47b41d88-a01f-4322-9eee-f2d304e40a11', TIMESTAMP '2026-09-25 08:35:30.973987', TIMESTAMP '2026-09-25 08:35:30.973996', U&'Task 2 \00b7 \2713', 'OPEN', NULL, NULL, NULL, NULL),
(UUID 'efed1d79-c6cd-464c-8f59-9495582dbeea', 't3', 'TODO', 'Descr 3', UUID '47b41d88-a01f-4322-9eee-f2d304e40a11', TIMESTAMP '2026-09-25 08:35:30.981601', TIMESTAMP '2026-09-25 08:35:30.981614', U&'Task 3 \00b7 \2713', 'OPEN', NULL, NULL, NULL, NULL),
(UUID '6b24f014-f9f9-4cb5-af6e-54e4fdd0eaaa', 't4', 'TODO', 'Descr 4', UUID '47b41d88-a01f-4322-9eee-f2d304e40a11', TIMESTAMP '2026-09-25 08:35:30.989672', TIMESTAMP '2026-09-25 08:35:30.989683', U&'Task 4 \00b7 \2713', 'OPEN', NULL, NULL, NULL, NULL),
(UUID '618af982-29de-4177-b330-e62424a97115', 't5', 'TODO', 'Descr 5', UUID '47b41d88-a01f-4322-9eee-f2d304e40a11', TIMESTAMP '2026-09-25 08:35:30.997021', TIMESTAMP '2026-09-25 08:35:30.99703', U&'Task 5 \00b7 \2713', 'OPEN', NULL, NULL, NULL, NULL),
(UUID 'be6b254e-55d3-43a4-a6ce-8ee3f4e47e5d', 't0', 'TODO', 'Descr 0', UUID '3c428e0b-9f6b-4f6c-bef2-4bd368136ddc', TIMESTAMP '2026-09-25 08:35:31.104486', TIMESTAMP '2026-09-25 08:35:31.104496', U&'Task 0 \00b7 \2713', 'OPEN', NULL, NULL, NULL, NULL);         
INSERT INTO "PUBLIC"."TASK" VALUES
(UUID '370d6431-960b-4c77-952d-34b376a2950d', 't1', 'TODO', 'Descr 1', UUID '3c428e0b-9f6b-4f6c-bef2-4bd368136ddc', TIMESTAMP '2026-09-25 08:35:31.112104', TIMESTAMP '2026-09-25 08:35:31.112113', U&'Task 1 \00b7 \2713', 'OPEN', NULL, NULL, NULL, NULL),
(UUID '4a734fa3-0a0e-4139-afc3-faf034a75ee5', 't2', 'TODO', 'Descr 2', UUID '3c428e0b-9f6b-4f6c-bef2-4bd368136ddc', TIMESTAMP '2026-09-25 08:35:31.118847', TIMESTAMP '2026-09-25 08:35:31.118854', U&'Task 2 \00b7 \2713', 'OPEN', NULL, NULL, NULL, NULL),
(UUID 'eb4dc1a0-95f3-41d8-bf28-144769f2c8e1', 't3', 'TODO', 'Descr 3', UUID '3c428e0b-9f6b-4f6c-bef2-4bd368136ddc', TIMESTAMP '2026-09-25 08:35:31.12587', TIMESTAMP '2026-09-25 08:35:31.12589', U&'Task 3 \00b7 \2713', 'OPEN', NULL, NULL, NULL, NULL),
(UUID '40d47d02-4102-45e0-8d20-c63f4537bb78', 't4', 'TODO', 'Descr 4', UUID '3c428e0b-9f6b-4f6c-bef2-4bd368136ddc', TIMESTAMP '2026-09-25 08:35:31.133468', TIMESTAMP '2026-09-25 08:35:31.133476', U&'Task 4 \00b7 \2713', 'OPEN', NULL, NULL, NULL, NULL),
(UUID '68857795-f358-448d-9c3d-8f11638ca39f', 't5', 'TODO', 'Descr 5', UUID '3c428e0b-9f6b-4f6c-bef2-4bd368136ddc', TIMESTAMP '2026-09-25 08:35:31.140454', TIMESTAMP '2026-09-25 08:35:31.140462', U&'Task 5 \00b7 \2713', 'OPEN', NULL, NULL, NULL, NULL);              
CREATE CACHED TABLE "PUBLIC"."RUN_QUEUE_ITEM"(
    "ID" UUID NOT NULL,
    "TASK_ID" UUID NOT NULL,
    "POSITION" INTEGER NOT NULL,
    "STATE" CHARACTER VARYING(20) NOT NULL,
    "DETAIL" CHARACTER VARYING(500),
    "STARTED_AT" TIMESTAMP,
    "FINISHED_AT" TIMESTAMP,
    "CREATED_AT" TIMESTAMP NOT NULL
);          
ALTER TABLE "PUBLIC"."RUN_QUEUE_ITEM" ADD CONSTRAINT "PUBLIC"."PK_RUN_QUEUE_ITEM" PRIMARY KEY("ID");           
-- 1 +/- SELECT COUNT(*) FROM PUBLIC.RUN_QUEUE_ITEM;           
INSERT INTO "PUBLIC"."RUN_QUEUE_ITEM" VALUES
(UUID 'c4121d7f-db1d-4736-950e-029591ffec73', UUID '2550ba76-9e87-4d75-8ab8-9ea96949d328', 0, 'QUEUED', NULL, NULL, NULL, TIMESTAMP '2026-09-25 08:35:30.261926');
CREATE INDEX "PUBLIC"."IX_RUN_QUEUE_ITEM_POSITION" ON "PUBLIC"."RUN_QUEUE_ITEM"("POSITION" NULLS FIRST);       
CREATE CACHED TABLE "PUBLIC"."TIME_ENTRY"(
    "ID" UUID NOT NULL,
    "TASK_ID" UUID NOT NULL,
    "STARTED_AT" TIMESTAMP NOT NULL,
    "STOPPED_AT" TIMESTAMP,
    "CREATED_AT" TIMESTAMP NOT NULL,
    "UPDATED_AT" TIMESTAMP NOT NULL
);   
ALTER TABLE "PUBLIC"."TIME_ENTRY" ADD CONSTRAINT "PUBLIC"."PK_TIME_ENTRY" PRIMARY KEY("ID");   
-- 1 +/- SELECT COUNT(*) FROM PUBLIC.TIME_ENTRY;               
INSERT INTO "PUBLIC"."TIME_ENTRY" VALUES
(UUID '005e923f-8717-4363-9ee1-33aa6cb2bc48', UUID '2550ba76-9e87-4d75-8ab8-9ea96949d328', TIMESTAMP '2026-01-01 09:00:00', TIMESTAMP '2026-01-01 10:30:00', TIMESTAMP '2026-09-25 08:35:29.879698', TIMESTAMP '2026-09-25 08:35:29.936311');         
CREATE CACHED TABLE "PUBLIC"."DOCUMENT_TASK"(
    "DOCUMENT_ID" UUID NOT NULL,
    "TASK_ID" UUID NOT NULL,
    "POSITION" INTEGER NOT NULL
); 
ALTER TABLE "PUBLIC"."DOCUMENT_TASK" ADD CONSTRAINT "PUBLIC"."PK_DOCUMENT_TASK" PRIMARY KEY("DOCUMENT_ID", "TASK_ID");         
-- 30 +/- SELECT COUNT(*) FROM PUBLIC.DOCUMENT_TASK;           
INSERT INTO "PUBLIC"."DOCUMENT_TASK" VALUES
(UUID '24305327-bfce-43c1-9884-87ce2b35e7e8', UUID '2550ba76-9e87-4d75-8ab8-9ea96949d328', 0),
(UUID '05ba302d-6119-4c76-94b6-35935c259ff5', UUID '2550ba76-9e87-4d75-8ab8-9ea96949d328', 1),
(UUID '05ba302d-6119-4c76-94b6-35935c259ff5', UUID '7aee22c5-55d1-47bf-aafa-102f57870d14', 0),
(UUID '25d59e00-bbe4-4e9b-b329-333dde4d9255', UUID '24865959-53df-4432-b815-e6d4ba549165', 0),
(UUID '25d59e00-bbe4-4e9b-b329-333dde4d9255', UUID 'dcc6e78e-ab4e-4948-8b39-c021fe9a4f43', 0),
(UUID '25d59e00-bbe4-4e9b-b329-333dde4d9255', UUID '826123bd-a656-4eb5-8051-bef2dc7a8685', 0),
(UUID '25d59e00-bbe4-4e9b-b329-333dde4d9255', UUID '770a6413-73fc-4004-b61d-843c2c4502ee', 0),
(UUID '25d59e00-bbe4-4e9b-b329-333dde4d9255', UUID '625ddb04-04f1-41e6-8326-3f22ff4e2ca3', 0),
(UUID '25d59e00-bbe4-4e9b-b329-333dde4d9255', UUID 'd0b43f69-6289-4e50-935c-f9af304cec87', 0),
(UUID 'aa5fb4b8-a98b-4130-b070-2456e855b71e', UUID '24865959-53df-4432-b815-e6d4ba549165', 1),
(UUID 'aa5fb4b8-a98b-4130-b070-2456e855b71e', UUID 'dcc6e78e-ab4e-4948-8b39-c021fe9a4f43', 1),
(UUID 'aa5fb4b8-a98b-4130-b070-2456e855b71e', UUID '826123bd-a656-4eb5-8051-bef2dc7a8685', 1),
(UUID '2758126a-6182-4945-9e1d-8738907b3031', UUID '43fff879-efaa-4b87-bbfd-8910ebaad234', 0),
(UUID '2758126a-6182-4945-9e1d-8738907b3031', UUID '6e3a5ee1-bda8-4dbc-b205-1f288b89bd92', 0),
(UUID '2758126a-6182-4945-9e1d-8738907b3031', UUID '70786d31-eda4-4e63-accc-ad19868291ae', 0),
(UUID '2758126a-6182-4945-9e1d-8738907b3031', UUID 'efed1d79-c6cd-464c-8f59-9495582dbeea', 0),
(UUID '2758126a-6182-4945-9e1d-8738907b3031', UUID '6b24f014-f9f9-4cb5-af6e-54e4fdd0eaaa', 0),
(UUID '2758126a-6182-4945-9e1d-8738907b3031', UUID '618af982-29de-4177-b330-e62424a97115', 0),
(UUID '63817584-17db-445a-8c2b-7d146d393380', UUID '43fff879-efaa-4b87-bbfd-8910ebaad234', 1),
(UUID '63817584-17db-445a-8c2b-7d146d393380', UUID '6e3a5ee1-bda8-4dbc-b205-1f288b89bd92', 1),
(UUID '63817584-17db-445a-8c2b-7d146d393380', UUID '70786d31-eda4-4e63-accc-ad19868291ae', 1),
(UUID '7cf60c0c-8817-44ac-8f47-d14662eaaa31', UUID 'be6b254e-55d3-43a4-a6ce-8ee3f4e47e5d', 0),
(UUID '7cf60c0c-8817-44ac-8f47-d14662eaaa31', UUID '370d6431-960b-4c77-952d-34b376a2950d', 0),
(UUID '7cf60c0c-8817-44ac-8f47-d14662eaaa31', UUID '4a734fa3-0a0e-4139-afc3-faf034a75ee5', 0),
(UUID '7cf60c0c-8817-44ac-8f47-d14662eaaa31', UUID 'eb4dc1a0-95f3-41d8-bf28-144769f2c8e1', 0),
(UUID '7cf60c0c-8817-44ac-8f47-d14662eaaa31', UUID '40d47d02-4102-45e0-8d20-c63f4537bb78', 0),
(UUID '7cf60c0c-8817-44ac-8f47-d14662eaaa31', UUID '68857795-f358-448d-9c3d-8f11638ca39f', 0),
(UUID 'ab6ffac0-a79c-4790-a58c-9b3e17737272', UUID 'be6b254e-55d3-43a4-a6ce-8ee3f4e47e5d', 1),
(UUID 'ab6ffac0-a79c-4790-a58c-9b3e17737272', UUID '370d6431-960b-4c77-952d-34b376a2950d', 1),
(UUID 'ab6ffac0-a79c-4790-a58c-9b3e17737272', UUID '4a734fa3-0a0e-4139-afc3-faf034a75ee5', 1);  
CREATE INDEX "PUBLIC"."IDX_DOCUMENT_TASK_TASK" ON "PUBLIC"."DOCUMENT_TASK"("TASK_ID" NULLS FIRST);             
CREATE CACHED TABLE "PUBLIC"."DOCUMENT"(
    "ID" UUID NOT NULL,
    "TITLE" CHARACTER VARYING(255) NOT NULL,
    "KIND" CHARACTER VARYING(40) NOT NULL,
    "BODY_MARKDOWN" CHARACTER VARYING(100000) NOT NULL,
    "SOURCE_PATH" CHARACTER VARYING(500),
    "CREATED_AT" TIMESTAMP NOT NULL,
    "UPDATED_AT" TIMESTAMP NOT NULL,
    "CONTEXT_MODE" CHARACTER VARYING(20) DEFAULT 'FULL' NOT NULL
);       
ALTER TABLE "PUBLIC"."DOCUMENT" ADD CONSTRAINT "PUBLIC"."PK_DOCUMENT" PRIMARY KEY("ID");       
-- 8 +/- SELECT COUNT(*) FROM PUBLIC.DOCUMENT; 
INSERT INTO "PUBLIC"."DOCUMENT" VALUES
(UUID '24305327-bfce-43c1-9884-87ce2b35e7e8', 'CONTEXT.md', 'context', U&'# Contesto\000a\000aIl workflow parte da POST /api/v1/pipelines.', NULL, TIMESTAMP '2026-09-25 08:35:29.082623', TIMESTAMP '2026-09-25 08:35:29.082637', 'FULL'),
(UUID '05ba302d-6119-4c76-94b6-35935c259ff5', 'kmaster14.md', 'notes', U&'Accesso via bastion.\000a\000aDettaglio.', NULL, TIMESTAMP '2026-09-25 08:35:29.108282', TIMESTAMP '2026-09-25 08:35:29.181757', 'REFERENCE'),
(UUID '25d59e00-bbe4-4e9b-b329-333dde4d9255', 'shared.md', 'notes', 'Condivisa fra sei task', NULL, TIMESTAMP '2026-09-25 08:35:30.859072', TIMESTAMP '2026-09-25 08:35:30.859084', 'FULL'),
(UUID 'aa5fb4b8-a98b-4130-b070-2456e855b71e', 'pair.md', 'notes', 'Tre task', NULL, TIMESTAMP '2026-09-25 08:35:30.875439', TIMESTAMP '2026-09-25 08:35:30.87545', 'FULL'),
(UUID '2758126a-6182-4945-9e1d-8738907b3031', 'shared.md', 'notes', 'Condivisa fra sei task', NULL, TIMESTAMP '2026-09-25 08:35:31.007748', TIMESTAMP '2026-09-25 08:35:31.007758', 'FULL'),
(UUID '63817584-17db-445a-8c2b-7d146d393380', 'pair.md', 'notes', 'Tre task', NULL, TIMESTAMP '2026-09-25 08:35:31.021871', TIMESTAMP '2026-09-25 08:35:31.021881', 'FULL'),
(UUID '7cf60c0c-8817-44ac-8f47-d14662eaaa31', 'shared.md', 'notes', 'Condivisa fra sei task', NULL, TIMESTAMP '2026-09-25 08:35:31.156759', TIMESTAMP '2026-09-25 08:35:31.15677', 'FULL'),
(UUID 'ab6ffac0-a79c-4790-a58c-9b3e17737272', 'pair.md', 'notes', 'Tre task', NULL, TIMESTAMP '2026-09-25 08:35:31.168773', TIMESTAMP '2026-09-25 08:35:31.168782', 'FULL');        
CREATE CACHED TABLE "PUBLIC"."TASK_STEP"(
    "ID" UUID NOT NULL,
    "TASK_ID" UUID NOT NULL,
    "TITLE" CHARACTER VARYING(200) NOT NULL,
    "BODY_MARKDOWN" CHARACTER VARYING(20000),
    "DONE_AT" TIMESTAMP,
    "POSITION" INTEGER NOT NULL,
    "CREATED_AT" TIMESTAMP NOT NULL,
    "UPDATED_AT" TIMESTAMP NOT NULL,
    "STATE" CHARACTER VARYING(20) DEFAULT 'DRAFT' NOT NULL,
    "RUNNING_AT" TIMESTAMP,
    "CLAIMED_AT" TIMESTAMP
);            
ALTER TABLE "PUBLIC"."TASK_STEP" ADD CONSTRAINT "PUBLIC"."PK_TASK_STEP" PRIMARY KEY("ID");     
-- 3 +/- SELECT COUNT(*) FROM PUBLIC.TASK_STEP;
INSERT INTO "PUBLIC"."TASK_STEP" VALUES
(UUID 'd41b7329-a679-4996-8528-bfa38e363709', UUID '2550ba76-9e87-4d75-8ab8-9ea96949d328', 'Aggregate the rows', 'Somma per settimana.', TIMESTAMP '2026-09-25 08:35:29.836744', 1, TIMESTAMP '2026-09-25 08:35:29.207459', TIMESTAMP '2026-09-25 08:35:29.837872', 'DONE', TIMESTAMP '2026-09-25 08:35:29.562919', TIMESTAMP '2026-09-25 08:35:29.562919'),
(UUID '2cb1fe72-27e5-4ac2-9e39-4f76df70f7ba', UUID '2550ba76-9e87-4d75-8ab8-9ea96949d328', 'Write the tests', NULL, NULL, 0, TIMESTAMP '2026-09-25 08:35:29.227645', TIMESTAMP '2026-09-25 08:35:29.586484', 'CLAIMED', TIMESTAMP '2026-09-25 08:35:29.54474', TIMESTAMP '2026-09-25 08:35:29.585676'),
(UUID '7381fcdf-488a-40c3-9f6d-7c221a9a8f06', UUID '2550ba76-9e87-4d75-8ab8-9ea96949d328', 'Expose the download', 'Stream it.', NULL, 2, TIMESTAMP '2026-09-25 08:35:29.620644', TIMESTAMP '2026-09-25 08:35:29.620654', 'DRAFT', NULL, NULL);    
CREATE INDEX "PUBLIC"."IDX_TASK_STEP_TASK" ON "PUBLIC"."TASK_STEP"("TASK_ID" NULLS FIRST, "POSITION" NULLS FIRST);             
CREATE CACHED TABLE "PUBLIC"."COMMIT_REFERENCE"(
    "ID" UUID NOT NULL,
    "TASK_ID" UUID NOT NULL,
    "STEP_ID" UUID,
    "COMMIT_HASH" CHARACTER VARYING(40) NOT NULL,
    "COMMENT" CHARACTER VARYING(200) NOT NULL,
    "CREATED_AT" TIMESTAMP NOT NULL,
    "DIFF" CHARACTER LARGE OBJECT,
    "IN_CONTEXT" BOOLEAN DEFAULT FALSE NOT NULL
);          
ALTER TABLE "PUBLIC"."COMMIT_REFERENCE" ADD CONSTRAINT "PUBLIC"."PK_COMMIT_REFERENCE" PRIMARY KEY("ID");       
-- 2 +/- SELECT COUNT(*) FROM PUBLIC.COMMIT_REFERENCE;         
INSERT INTO "PUBLIC"."COMMIT_REFERENCE" VALUES
(UUID '87c495f4-1853-432c-90f9-9aad6da8db8a', UUID '2550ba76-9e87-4d75-8ab8-9ea96949d328', NULL, '25986f428c36f462589a8f649b700985256f0754', 'Wire up the ledger', TIMESTAMP '2026-09-25 08:35:29.714445', U&'diff --git a/README.md b/README.md\000aindex 5626abf..814f4a4 100644\000a--- a/README.md\000a+++ b/README.md\000a@@ -1 +1,2 @@\000a one\000a+two', FALSE),
(UUID '592145dd-1204-4a9d-b30e-fea779477e34', UUID '2550ba76-9e87-4d75-8ab8-9ea96949d328', UUID '2cb1fe72-27e5-4ac2-9e39-4f76df70f7ba', '25986f428c36f462589a8f649b700985256f0754', 'Wire up the ledger', TIMESTAMP '2026-09-25 08:35:29.743417', U&'diff --git a/README.md b/README.md\000aindex 5626abf..814f4a4 100644\000a--- a/README.md\000a+++ b/README.md\000a@@ -1 +1,2 @@\000a one\000a+two', FALSE);        
CREATE CACHED TABLE "PUBLIC"."PROJECT"(
    "ID" UUID NOT NULL,
    "LABEL" CHARACTER VARYING(120) NOT NULL,
    "STATUS" CHARACTER VARYING(20) NOT NULL,
    "DESCRIPTION" CHARACTER VARYING(100000),
    "CREATED_AT" TIMESTAMP NOT NULL,
    "UPDATED_AT" TIMESTAMP NOT NULL,
    "COMPANY_ID" UUID NOT NULL,
    "TITLE" CHARACTER VARYING(200) NOT NULL,
    "BLUEPRINT_MARKDOWN" CHARACTER VARYING(100000),
    "REPO_FOLDER" CHARACTER VARYING(1000),
    "AUTO_COMMIT" BOOLEAN DEFAULT FALSE NOT NULL
);               
ALTER TABLE "PUBLIC"."PROJECT" ADD CONSTRAINT "PUBLIC"."PK_PROJECT" PRIMARY KEY("ID");         
-- 5 +/- SELECT COUNT(*) FROM PUBLIC.PROJECT;  
INSERT INTO "PUBLIC"."PROJECT" VALUES
(UUID 'bfc1be35-3de7-4cd7-ad87-ce6ec42674f5', 'vega-platform', 'ACTIVE', 'Progetto', TIMESTAMP '2026-09-25 08:35:28.736529', TIMESTAMP '2026-09-25 08:35:28.736557', UUID '80ba5f6b-b765-4a47-8487-14d2356de37e', 'Vega', NULL, '/srv/rekall/repo', FALSE),
(UUID '58ddefe2-ed26-4366-94b0-ff7154dd1746', 'beacon', 'DONE', NULL, TIMESTAMP '2026-09-25 08:35:28.751878', TIMESTAMP '2026-09-25 08:35:28.878244', UUID '80ba5f6b-b765-4a47-8487-14d2356de37e', 'Beacon 2', '# Blueprint', '/srv/rekall/plain', FALSE),
(UUID '88ba951a-8d7b-4901-a68b-8e90c0e85fa6', 'p9b94b2', 'ACTIVE', NULL, TIMESTAMP '2026-09-25 08:35:30.779778', TIMESTAMP '2026-09-25 08:35:30.779788', UUID '4c231b5f-44bd-4468-8089-59a982ab5243', U&'Progetto \00e8 \00fc', NULL, NULL, FALSE),
(UUID '47b41d88-a01f-4322-9eee-f2d304e40a11', 'p318612', 'ACTIVE', NULL, TIMESTAMP '2026-09-25 08:35:30.951106', TIMESTAMP '2026-09-25 08:35:30.951116', UUID '255cfb53-c59f-43eb-a11b-ea632f186067', U&'Progetto \00e8 \00fc', NULL, NULL, FALSE),
(UUID '3c428e0b-9f6b-4f6c-bef2-4bd368136ddc', 'pfc98f5', 'ACTIVE', NULL, TIMESTAMP '2026-09-25 08:35:31.096137', TIMESTAMP '2026-09-25 08:35:31.096145', UUID '6174fb70-074b-4331-aa61-7fd215d86275', U&'Progetto \00e8 \00fc', NULL, NULL, FALSE);       
ALTER TABLE "PUBLIC"."TASK" ADD CONSTRAINT "PUBLIC"."UQ_TASK_PROJECT_LABEL" UNIQUE NULLS DISTINCT ("PROJECT_ID", "LABEL");     
ALTER TABLE "PUBLIC"."TAG" ADD CONSTRAINT "PUBLIC"."UQ_TAG_NAME" UNIQUE NULLS DISTINCT ("NAME");               
ALTER TABLE "PUBLIC"."PROJECT" ADD CONSTRAINT "PUBLIC"."UQ_PROJECT_COMPANY_LABEL" UNIQUE NULLS DISTINCT ("COMPANY_ID", "LABEL");               
ALTER TABLE "PUBLIC"."WRAPUP" ADD CONSTRAINT "PUBLIC"."UQ_WRAPUP_TASK" UNIQUE NULLS DISTINCT ("TASK_ID");      
ALTER TABLE "PUBLIC"."COMPANY" ADD CONSTRAINT "PUBLIC"."UQ_COMPANY_NAME" UNIQUE NULLS DISTINCT ("NAME");       
ALTER TABLE "PUBLIC"."TASK" ADD CONSTRAINT "PUBLIC"."FK_TASK_PROJECT" FOREIGN KEY("PROJECT_ID") REFERENCES "PUBLIC"."PROJECT"("ID") ON DELETE CASCADE NOCHECK; 
ALTER TABLE "PUBLIC"."TASK" ADD CONSTRAINT "PUBLIC"."FK_TASK_TAG" FOREIGN KEY("TAG_ID") REFERENCES "PUBLIC"."TAG"("ID") ON DELETE SET NULL NOCHECK;            
ALTER TABLE "PUBLIC"."RUN_QUEUE_ITEM" ADD CONSTRAINT "PUBLIC"."FK_RUN_QUEUE_ITEM_TASK" FOREIGN KEY("TASK_ID") REFERENCES "PUBLIC"."TASK"("ID") ON DELETE CASCADE NOCHECK;      
ALTER TABLE "PUBLIC"."TASK_REVISION" ADD CONSTRAINT "PUBLIC"."FK_TASK_REVISION_TASK" FOREIGN KEY("TASK_ID") REFERENCES "PUBLIC"."TASK"("ID") ON DELETE CASCADE NOCHECK;        
ALTER TABLE "PUBLIC"."COMMIT_REFERENCE" ADD CONSTRAINT "PUBLIC"."FK_COMMIT_REFERENCE_TASK" FOREIGN KEY("TASK_ID") REFERENCES "PUBLIC"."TASK"("ID") ON DELETE CASCADE NOCHECK;  
ALTER TABLE "PUBLIC"."PROJECT" ADD CONSTRAINT "PUBLIC"."FK_PROJECT_COMPANY" FOREIGN KEY("COMPANY_ID") REFERENCES "PUBLIC"."COMPANY"("ID") ON DELETE CASCADE NOCHECK;           
ALTER TABLE "PUBLIC"."COMMIT_REFERENCE" ADD CONSTRAINT "PUBLIC"."FK_COMMIT_REFERENCE_STEP" FOREIGN KEY("STEP_ID") REFERENCES "PUBLIC"."TASK_STEP"("ID") ON DELETE CASCADE NOCHECK;             
ALTER TABLE "PUBLIC"."TASK_STEP" ADD CONSTRAINT "PUBLIC"."FK_TASK_STEP_TASK" FOREIGN KEY("TASK_ID") REFERENCES "PUBLIC"."TASK"("ID") ON DELETE CASCADE NOCHECK;
ALTER TABLE "PUBLIC"."DOCUMENT_TASK" ADD CONSTRAINT "PUBLIC"."FK_DOCUMENT_TASK_DOCUMENT" FOREIGN KEY("DOCUMENT_ID") REFERENCES "PUBLIC"."DOCUMENT"("ID") ON DELETE CASCADE NOCHECK;            
ALTER TABLE "PUBLIC"."DOCUMENT_TASK" ADD CONSTRAINT "PUBLIC"."FK_DOCUMENT_TASK_TASK" FOREIGN KEY("TASK_ID") REFERENCES "PUBLIC"."TASK"("ID") ON DELETE CASCADE NOCHECK;        
ALTER TABLE "PUBLIC"."WRAPUP" ADD CONSTRAINT "PUBLIC"."FK_WRAPUP_TASK" FOREIGN KEY("TASK_ID") REFERENCES "PUBLIC"."TASK"("ID") ON DELETE CASCADE NOCHECK;      
ALTER TABLE "PUBLIC"."TIME_ENTRY" ADD CONSTRAINT "PUBLIC"."FK_TIME_ENTRY_TASK" FOREIGN KEY("TASK_ID") REFERENCES "PUBLIC"."TASK"("ID") ON DELETE CASCADE NOCHECK;              
