This folder contains all the scripts I used to set up and populate the database.

[dbsetup.py](./dbsetup.py) was used to set up the schema in the database by executing the SQL file.

[schema.sql](./schema.sql) writes the tables for seasons, episodes, speakers, and lines.

[dbwriter.py](./dbwriter.py) loops through all seasons in the "Seasons" folder, and for each episode .txt file, parses it to get the episode number + title, loops through all lines to get speaker  data and the content of the line, then adds this data to the tables.
