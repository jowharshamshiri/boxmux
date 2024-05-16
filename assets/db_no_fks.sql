CREATE SEQUENCE IF NOT EXISTS seq_a START 1

--data type tables
CREATE TABLE IF NOT EXISTS data_types (id INTEGER PRIMARY KEY DEFAULT nextval('seq_a'), type TEXT)

CREATE TABLE IF NOT EXISTS maps (id INTEGER PRIMARY KEY DEFAULT nextval('seq_a'), map_name TEXT, data_type_id INTEGER)
CREATE TABLE IF NOT EXISTS maps_data (map_id INTEGER, key TEXT, value TEXT, idx INTEGER, PRIMARY KEY (map_id, key))

--list tables
CREATE TABLE IF NOT EXISTS lists (id INTEGER PRIMARY KEY DEFAULT nextval('seq_a'), list_name TEXT, data_type_id INTEGER)
CREATE TABLE IF NOT EXISTS lists_data (list_id INTEGER, value TEXT, idx INTEGER, PRIMARY KEY (list_id, idx))

--class tables
CREATE TABLE IF NOT EXISTS classes (id INTEGER PRIMARY KEY DEFAULT nextval('seq_a'), class_name TEXT)
CREATE TABLE IF NOT EXISTS classes_properties (id INTEGER PRIMARY KEY DEFAULT nextval('seq_a'), class_id INTEGER, property TEXT, data_type_id INTEGER)
CREATE TABLE IF NOT EXISTS classes_instances (class_id INTEGER, instance_id INTEGER UNIQUE DEFAULT nextval('seq_a'), PRIMARY KEY (class_id, instance_id), idx INTEGER)
CREATE TABLE IF NOT EXISTS classes_instances_data (instance_id INTEGER, property_id INTEGER, value TEXT, PRIMARY KEY (instance_id, property_id))

--set tables
CREATE TABLE IF NOT EXISTS sets (id INTEGER PRIMARY KEY DEFAULT nextval('seq_a'), set_name TEXT, data_type_id INTEGER)
CREATE TABLE IF NOT EXISTS sets_data (set_id INTEGER, value TEXT, PRIMARY KEY (set_id, value))

--queue tables
CREATE TABLE IF NOT EXISTS queues (id INTEGER PRIMARY KEY DEFAULT nextval('seq_a'), queue_name TEXT, data_type_id INTEGER)
CREATE TABLE IF NOT EXISTS queues_data (queue_id INTEGER, value TEXT, idx INTEGER, PRIMARY KEY (queue_id, idx))

--multimap tables
CREATE TABLE IF NOT EXISTS multimaps (id INTEGER PRIMARY KEY DEFAULT nextval('seq_a'), multimap_name TEXT, data_type_id INTEGER)
CREATE TABLE IF NOT EXISTS multimaps_data (multimap_id INTEGER, key TEXT, value TEXT, idx INTEGER, PRIMARY KEY (multimap_id, key, idx))

--stack tables
CREATE TABLE IF NOT EXISTS stacks (id INTEGER PRIMARY KEY DEFAULT nextval('seq_a'), stack_name TEXT, data_type_id INTEGER)
CREATE TABLE IF NOT EXISTS stacks_data (stack_id INTEGER, value TEXT, idx INTEGER, PRIMARY KEY (stack_id, idx))

--download tables
CREATE TABLE IF NOT EXISTS downloads (id INTEGER PRIMARY KEY DEFAULT nextval('seq_a'), url TEXT, expected_checksum TEXT, checksum TEXT, hash_type TEXT, downloaded BOOLEAN, download_time TIMESTAMP)
CREATE TABLE IF NOT EXISTS downloads_links (download_id INTEGER, link_path TEXT, PRIMARY KEY (download_id, link_path))

--settings table
CREATE TABLE IF NOT EXISTS settings (id INTEGER PRIMARY KEY DEFAULT nextval('seq_a'), key TEXT UNIQUE, data_type_id INTEGER, value TEXT)
