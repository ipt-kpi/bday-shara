import pandas as pd


def create_df(file, table, data):
    df = pd.DataFrame(data)
    df.index = df.index + 1

    query = f"INSERT INTO {table}(name) VALUES\n"
    for index, row in df.iterrows():
        name = row[0].replace("'", "''")
        query += f"('{name}'),\n"
    query = query[:-2] + ";\n"
    file.write(query)
    return df


def get_index(df, value):
    return df[df[0] == value].index[0]


if __name__ == '__main__':
    df = pd.read_csv("data.csv", header=None)
    with open("import_prizes.sql", "w+", encoding="utf-8") as file:
        teachers = create_df(file, "teacher", df[0].unique())

        subjects = create_df(file, "subject", df[1].unique())

        prizes_types = create_df(file, "prize_type", df[2].unique())

        group_list = set()
        for groups in df[3].unique():
            for group in groups.split(", "):
                group_list.add(group)
        groups = create_df(file, "group_code", group_list)

        prizes = df[[0, 1, 2, 4]].drop_duplicates()
        for index, row in teachers.iterrows():
            prizes.loc[(prizes[0] == row[0]), 0] = get_index(teachers, row[0])

        for index, row in subjects.iterrows():
            prizes.loc[(prizes[1] == row[0]), 1] = get_index(subjects, row[0])

        for index, row in prizes_types.iterrows():
            prizes.loc[(prizes[2] == row[0]), 2] = get_index(prizes_types, row[0])

        prizes = prizes.reset_index(drop=True)
        prizes.index = prizes.index + 1

        query = f"INSERT INTO prize(teacher, subject, type, count) VALUES\n"
        for index, row in prizes.iterrows():
            query += f"({row[0]}, {row[1]}, {row[2]}, {row[4]}),\n"
        query = query[:-2] + ";\n"
        file.write(query)

        query_group = f"INSERT INTO prize_group(prize, group_code) VALUES\n"
        query_multiple_group = f"INSERT INTO prize_multiple_group(prize, group_code) VALUES\n"
        for index, row in df.iterrows():
            teacher = get_index(teachers, row[0])
            subject = get_index(subjects, row[1])
            prizes_type = get_index(prizes_types, row[2])
            count = row[4]
            group_list = row[3].split(", ")

            prize_id = prizes[
                (prizes[0] == teacher) &
                (prizes[1] == subject) &
                (prizes[2] == prizes_type) &
                (prizes[4] == count)
                ].index[0]

            if len(group_list) == 1:
                group_code = get_index(groups, row[3])
                query_group += f"({prize_id}, {group_code}),\n"
            else:
                for group in group_list:
                    group_code = get_index(groups, group)
                    query_multiple_group += f"({prize_id}, {group_code}),\n"
        query_group = query_group[:-2] + ";\n"
        query_multiple_group = query_multiple_group[:-2] + ";\n"
        file.write(query_group)
        file.write(query_multiple_group)
