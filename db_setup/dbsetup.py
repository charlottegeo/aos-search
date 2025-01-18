import psycopg2
import dotenv
import os

dotenv.load_dotenv()

USERNAME = os.getenv("USERNAME")
PASSWORD = os.getenv("PASSWORD")
DATABASE_NAME = os.getenv("DATABASE_NAME")
DATABASE_URL = os.getenv("DATABASE_URL")

conn = psycopg2.connect(dbname=DATABASE_NAME, user=USERNAME, password=PASSWORD, host=DATABASE_URL)
cursor = conn.cursor()

with open("schema.sql", "r") as file:
    sql_script = file.read()

cursor.execute(sql_script)
conn.commit()

#This just prints all the tables to make sure the sql script worked
cursor.execute("SELECT table_name FROM information_schema.tables WHERE table_schema = 'public'")
table_names = cursor.fetchall()
for table_name in table_names:
    print(table_name[0])

cursor.close()
conn.close()
