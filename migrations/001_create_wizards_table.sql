CREATE TABLE wizards(
  id SERIAL PRIMARY KEY,
  name VARCHAR NOT NULL,
  title VARCHAR NOT NULL,
  age INT NOT NULL,
  image_name VARCHAR
);

CREATE INDEX unique_wizards_index ON wizards (id);

