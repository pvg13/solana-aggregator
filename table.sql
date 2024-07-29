CREATE TABLE transactions (
    id SERIAL PRIMARY KEY,
    sender VARCHAR(44),
    receiver VARCHAR(44),
    amount BIGINT,
    timestamp TIMESTAMPTZ
);

CREATE TABLE accounts (
    pubkey VARCHAR(44) PRIMARY KEY,
    lamports BIGINT,
    owner VARCHAR(44),
    executable BOOLEAN,
    rent_epoch BIGINT
);
