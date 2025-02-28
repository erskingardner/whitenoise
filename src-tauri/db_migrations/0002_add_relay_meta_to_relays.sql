-- Add relay_meta column to account_relays and group_relays tables
ALTER TABLE account_relays ADD COLUMN relay_meta TEXT NOT NULL DEFAULT 'ReadWrite' CHECK (relay_meta IN ('Read', 'Write', 'ReadWrite'));
ALTER TABLE group_relays ADD COLUMN relay_meta TEXT NOT NULL DEFAULT 'ReadWrite' CHECK (relay_meta IN ('Read', 'Write', 'ReadWrite'));

-- Add an index to improve query performance when filtering by relay_meta
CREATE INDEX idx_account_relays_meta ON account_relays(relay_meta);
CREATE INDEX idx_group_relays_meta ON group_relays(relay_meta);
