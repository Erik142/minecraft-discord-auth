CREATE TABLE IF NOT EXISTS Players (
       discordName TEXT PRIMARY KEY,
       minecraftName TEXT UNIQUE,
       registrationCode TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS AuthenticationRequests(
       id SERIAL PRIMARY KEY,
       minecraftName TEXT NOT NULL,
       minecraftServer TEXT NOT NULL,
       ipAddress TEXT NOT NULL,
       handled BOOLEAN NOT NULL DEFAULT FALSE,
       created TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
       FOREIGN KEY (minecraftName) REFERENCES Players(minecraftName) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS PlayerAuthentications (
       id SERIAL,
       authRequestId INT,
       discordName TEXT NOT NULL UNIQUE,
       expiration TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP + (30 * INTERVAL '1 minute'),
       PRIMARY KEY (id),
       FOREIGN KEY (discordName) REFERENCES Players(discordName) ON DELETE CASCADE ON UPDATE CASCADE,
       FOREIGN KEY (authRequestId) REFERENCES AuthenticationRequests(id) ON DELETE CASCADE ON UPDATE CASCADE
);
