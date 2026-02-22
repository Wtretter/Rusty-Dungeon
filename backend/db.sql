-- database schema
-- table - users
--     -username
--     <!-- -pw hash -->
--     -money
--     -character stats
--         -hp
--         -damage
--         -luck
--         -resistance
--         -crit%

-- table - enemies
--     -name
--     -hp
--     -damage
--     -resistance
--     -crit%
--     -value
--     <!-- specials -->

-- DROP TABLE IF EXISTS users;
-- DROP TABLE IF EXISTS enemies;

CREATE TABLE users (
    id serial PRIMARY KEY,
    username text, 
    money integer, 
    hitpoints integer, 
    damage integer, 
    luck integer, 
    resistance integer, 
    crit integer
);

CREATE TABLE enemies (
    id serial PRIMARY KEY,
    name text,
    hitpoints integer,
    damage integer,
    resistance integer,
    crit integer,
    value integer
);

INSERT INTO enemies (name, hitpoints, damage, resistance, crit, value)
VALUES ('Ford', 1, 1, 0, 0, 1), ('Dodge', 1, 1, 1, 0, 2), ('Cadillac', 5, 5, 2, 5, 10);