\set QUIET true
SET client_min_messages TO WARNING;
DROP SCHEMA public CASCADE;
CREATE SCHEMA public;
GRANT ALL ON SCHEMA public TO postgres;
\set QUIET false

\ir tables.sql
\ir views.sql
\ir triggers.sql
