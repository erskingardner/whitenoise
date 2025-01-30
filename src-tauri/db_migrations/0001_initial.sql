-- Accounts table with JSON fields for complex objects
CREATE TABLE accounts (
    pubkey TEXT PRIMARY KEY,
    metadata TEXT NOT NULL,  -- JSON string for nostr Metadata
    settings TEXT NOT NULL,  -- JSON string for AccountSettings
    onboarding TEXT NOT NULL,  -- JSON string for AccountOnboarding
    last_used INTEGER NOT NULL,
    last_synced INTEGER NOT NULL,
    active BOOLEAN NOT NULL DEFAULT FALSE
);

-- Create an index for faster lookups
CREATE INDEX idx_accounts_active ON accounts(active);

-- Create a unique partial index that only allows one TRUE value
CREATE UNIQUE INDEX idx_accounts_single_active ON accounts(active) WHERE active = TRUE;

-- Create a trigger to ensure only one active account
CREATE TRIGGER ensure_single_active_account
   BEFORE UPDATE ON accounts
   WHEN NEW.active = TRUE
BEGIN
    UPDATE accounts SET active = FALSE WHERE active = TRUE AND pubkey != NEW.pubkey;
END;

-- Account-specific relays table
CREATE TABLE account_relays (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    url TEXT NOT NULL,
    relay_type TEXT NOT NULL,
    account_pubkey TEXT NOT NULL,
    FOREIGN KEY (account_pubkey) REFERENCES accounts(pubkey) ON DELETE CASCADE
);

CREATE INDEX idx_account_relays_account ON account_relays(account_pubkey, relay_type);

-- Group-specific relays table (separate table)
CREATE TABLE group_relays (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    url TEXT NOT NULL,
    relay_type TEXT NOT NULL,
    account_pubkey TEXT NOT NULL,
    group_id BLOB NOT NULL,
    FOREIGN KEY (group_id, account_pubkey) REFERENCES groups(mls_group_id, account_pubkey) ON DELETE CASCADE
);

CREATE INDEX idx_group_relays_group ON group_relays(group_id, relay_type);
CREATE INDEX idx_group_relays_group_account ON group_relays(group_id, account_pubkey);

-- Groups table matching the Group struct
CREATE TABLE groups (
    mls_group_id BLOB,
    account_pubkey TEXT NOT NULL,
    nostr_group_id TEXT NOT NULL,
    name TEXT,
    description TEXT,
    admin_pubkeys TEXT NOT NULL,  -- JSON array of strings
    last_message_id TEXT,
    last_message_at INTEGER,
    group_type TEXT NOT NULL CHECK (group_type IN ('DirectMessage', 'Group')),
    epoch INTEGER NOT NULL,
    state TEXT NOT NULL CHECK (state IN ('Active', 'Inactive')),
    FOREIGN KEY (account_pubkey) REFERENCES accounts(pubkey) ON DELETE CASCADE,
    PRIMARY KEY (mls_group_id, account_pubkey)
);

CREATE INDEX idx_groups_mls_group_id ON groups(mls_group_id);
CREATE INDEX idx_groups_account ON groups(account_pubkey);
CREATE INDEX idx_groups_nostr ON groups(nostr_group_id);
CREATE INDEX idx_groups_mls_group_id_account ON groups(mls_group_id, account_pubkey);

-- Invites table matching the Invite struct
CREATE TABLE invites (
    event_id TEXT PRIMARY KEY, -- the event_id of the 444 unsigned invite event
    account_pubkey TEXT NOT NULL,
    event TEXT NOT NULL,  -- JSON string for UnsignedEvent
    mls_group_id BLOB NOT NULL,
    nostr_group_id TEXT NOT NULL,
    group_name TEXT NOT NULL,
    group_description TEXT NOT NULL,
    group_admin_pubkeys TEXT NOT NULL,  -- JSON array of strings
    group_relays TEXT NOT NULL,         -- JSON array of strings
    inviter TEXT NOT NULL,
    member_count INTEGER NOT NULL,
    state TEXT NOT NULL,
    outer_event_id TEXT,  -- the event_id of the 1059 event that contained the invite

    FOREIGN KEY (account_pubkey) REFERENCES accounts(pubkey) ON DELETE CASCADE
);

CREATE INDEX idx_invites_mls_group ON invites(mls_group_id);
CREATE INDEX idx_invites_state ON invites(state);
CREATE INDEX idx_invites_account ON invites(account_pubkey);
CREATE INDEX idx_invites_outer_event_id ON invites(outer_event_id);
CREATE INDEX idx_invites_event_id ON invites(event_id);

CREATE TABLE processed_invites (
    event_id TEXT PRIMARY KEY, -- This is the outer event id of the 1059 gift wrap event
    invite_event_id TEXT, -- This is the event id of the 444 invite event
    account_pubkey TEXT NOT NULL, -- This is the pubkey of the account that processed the invite
    processed_at INTEGER NOT NULL, -- This is the timestamp of when the invite was processed
    state TEXT NOT NULL, -- This is the state of the invite processing
    failure_reason TEXT, -- This is the reason the invite failed to process

    FOREIGN KEY (account_pubkey) REFERENCES accounts(pubkey) ON DELETE CASCADE
);

CREATE INDEX idx_processed_invites_invite_event_id ON processed_invites(invite_event_id);

-- Messages table with full-text search
CREATE TABLE messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_id TEXT NOT NULL,
    mls_group_id BLOB NOT NULL,
    account_pubkey TEXT NOT NULL,
    author_pubkey TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    content TEXT NOT NULL,
    tags TEXT,  -- JSON array of nostr tags
    event TEXT NOT NULL,  -- JSON string for UnsignedEvent
    outer_event_id TEXT NOT NULL,  -- the event_id of the 445 event
    FOREIGN KEY (mls_group_id, account_pubkey) REFERENCES groups(mls_group_id, account_pubkey) ON DELETE CASCADE,
    FOREIGN KEY (account_pubkey) REFERENCES accounts(pubkey) ON DELETE CASCADE,
    UNIQUE(event_id, account_pubkey)  -- Ensure event_id is unique per account
);

CREATE INDEX idx_messages_group_time ON messages(mls_group_id, created_at);
CREATE INDEX idx_messages_account_time ON messages(account_pubkey, created_at);
CREATE INDEX idx_messages_author_time ON messages(author_pubkey, created_at);
CREATE INDEX idx_messages_outer_event_id ON messages(outer_event_id);
CREATE INDEX idx_messages_event_id ON messages(event_id);
CREATE INDEX idx_messages_event_id_account ON messages(event_id, account_pubkey);

-- Update processed_messages table to reference the new messages table structure
CREATE TABLE processed_messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_id TEXT NOT NULL, -- This is the outer event id of the 445 event
    message_event_id TEXT NOT NULL, -- This is the inner UnsignedEvent's id. This is the id of the events stored in the messages table.
    account_pubkey TEXT NOT NULL, -- This is the pubkey of the account that processed the message
    processed_at INTEGER NOT NULL, -- This is the timestamp of when the message was processed
    state TEXT NOT NULL, -- This is the state of the message processing
    failure_reason TEXT, -- This is the reason the message failed to process
    UNIQUE(event_id, account_pubkey),
    FOREIGN KEY (account_pubkey) REFERENCES accounts(pubkey) ON DELETE CASCADE
);

CREATE INDEX idx_processed_messages_message_event_id ON processed_messages(message_event_id);
CREATE INDEX idx_processed_messages_event_id_account ON processed_messages(event_id, account_pubkey);

-- Full-text search for messages
CREATE VIRTUAL TABLE messages_fts USING fts5(
    content,
    id UNINDEXED,  -- Change to reference the new id column
    content='messages'
);

-- Update FTS triggers to use id instead of event_id
CREATE TRIGGER messages_ai AFTER INSERT ON messages BEGIN
    INSERT INTO messages_fts(content, id) VALUES (new.content, new.id);
END;

CREATE TRIGGER messages_ad AFTER DELETE ON messages BEGIN
    INSERT INTO messages_fts(messages_fts, content, id) VALUES('delete', old.content, old.id);
END;

CREATE TRIGGER messages_au AFTER UPDATE ON messages BEGIN
    INSERT INTO messages_fts(messages_fts, content, id) VALUES('delete', old.content, old.id);
    INSERT INTO messages_fts(content, id) VALUES (new.content, new.id);
END;
