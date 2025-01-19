This folder contains all the scripts I used to set up and populate the database.

[schema.sql](./schema.sql) writes the tables for seasons, episodes, speakers, and lines.

[dbmaker.py](./dbmaker.py) executes the SQL file and makes the database, loops through all seasons in the "Seasons" folder, and for each episode .txt file, parses it to get the episode number + title, loops through all lines to get speaker data and the content of the line, then adds this data to the tables.
