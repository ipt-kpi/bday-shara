CREATE TABLE group_code(
    id SERIAL PRIMARY KEY,
    name VARCHAR(7) UNIQUE
);

CREATE TABLE student(
    id SERIAL PRIMARY KEY,
    chat_id BIGINT UNIQUE,
    username VARCHAR(255),
    last_name VARCHAR(35),
    first_name VARCHAR(35),
    patronymic VARCHAR(35),
    group_code INTEGER REFERENCES group_code,
    UNIQUE(last_name, first_name, patronymic)
);

CREATE TABLE teacher(
    id SERIAL PRIMARY KEY,
    name VARCHAR(70) UNIQUE
);

CREATE TABLE subject(
    id SERIAL PRIMARY KEY,
    name VARCHAR(90) UNIQUE
);

CREATE TABLE prize_type(
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) UNIQUE
);

CREATE TABLE prize(
    id SERIAL PRIMARY KEY,
    teacher INTEGER REFERENCES teacher(id),
    subject INTEGER REFERENCES subject(id),
    type INTEGER REFERENCES prize_type(id),
    count SMALLINT CONSTRAINT count_check CHECK (count >= 1) DEFAULT 1
);

CREATE TABLE prize_group(
    prize INTEGER REFERENCES prize(id),
    group_code INTEGER REFERENCES group_code(id),
    PRIMARY KEY(prize, group_code)
);

CREATE TABLE prize_multiple_group(
    prize INTEGER REFERENCES prize(id),
    group_code INTEGER REFERENCES group_code(id),
    PRIMARY KEY(prize, group_code)
);

CREATE TYPE prize_status AS ENUM('expired', 'wait', 'success');

CREATE TABLE prize_student(
    prize INTEGER REFERENCES prize(id),
    student INTEGER REFERENCES student(id),
    PRIMARY KEY(prize, student)
);

CREATE TABLE prize_roll(
    prize INTEGER REFERENCES prize(id),
    student INTEGER REFERENCES student(id),
    status prize_status,
    signature VARCHAR(128)
);

CREATE TABLE teloxide_dialogues(
    chat_id BIGINT PRIMARY KEY,
    dialogue BYTEA NOT NULL
);

CREATE OR REPLACE FUNCTION mark_prize(
    chat_id_t BIGINT,
    prize_id INTEGER
) RETURNS void
AS $$
DECLARE
    exists bool = exists(
        SELECT * FROM student
        JOIN prize_group ON prize_group.group_code = student.group_code
        LEFT JOIN prize_multiple_group ON prize_group.group_code = student.group_code
        WHERE chat_id = chat_id_t AND (prize_group.prize = prize_id OR prize_multiple_group.prize = prize_id)
    );
    student_id int;
BEGIN
    SELECT id INTO student_id FROM student WHERE chat_id = chat_id_t;
    IF exists THEN
        IF (exists(SELECT * FROM prize_student WHERE prize = prize_id AND student = student_id)) THEN
            DELETE FROM prize_student WHERE prize = prize_id AND student = student_id;
        ELSE
            INSERT INTO prize_student(prize, student) VALUES (prize_id, student_id);
        END IF;
    END IF;
END $$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION get_prizes(chat_id_t BIGINT)
    RETURNS TABLE(
                     id INTEGER,
                     teacher CHARACTER VARYING,
                     subject CHARACTER VARYING,
                     prize_type CHARACTER VARYING,
                     count SMALLINT,
                     groups TEXT,
                     selected BOOLEAN
                 )
    LANGUAGE plpgsql
AS
$$
DECLARE
    student_id_t int;
    group_code_t int;
BEGIN
    SELECT student.id, student.group_code INTO student_id_t, group_code_t FROM student WHERE chat_id = chat_id_t;
    RETURN QUERY
        SELECT prizes.id, prizes.teacher, prizes.subject, prizes.prize_type, prizes.count, prizes.groups, (prize_student.student IS NOT NULL) AS selected FROM
            (SELECT prize.id AS id, teacher.name AS teacher, subject.name AS subject, prize_type.name AS prize_type, prize.count, null AS groups
             FROM prize_group
                      JOIN prize ON prize.id = prize_group.prize
                      JOIN teacher ON prize.teacher = teacher.id
                      JOIN subject ON prize.subject = subject.id
                      JOIN prize_type ON prize.type = prize_type.id
             WHERE group_code = group_code_t
             UNION
             SELECT prize.id, prize.teacher, prize.subject, prize.prize_type, prize.count, string_agg(distinct group_code.name, ', ' order by group_code.name) as groups
             FROM (
                      SELECT prize.id AS id, teacher.name AS teacher, subject.name AS subject, prize_type.name AS prize_type, prize.count, group_code
                      FROM prize_multiple_group
                               JOIN prize ON prize.id = prize_multiple_group.prize
                               JOIN teacher ON prize.teacher = teacher.id
                               JOIN subject ON prize.subject = subject.id
                               JOIN prize_type ON prize.type = prize_type.id
                      WHERE prize.id IN (SELECT prize FROM prize_multiple_group WHERE group_code = group_code_t)
                  ) as "prize"
                      JOIN group_code ON group_code.id = prize.group_code
             GROUP BY prize.id, prize.teacher, prize.subject, prize.prize_type, prize.count) AS prizes
                LEFT JOIN prize_student ON prize_student.prize = prizes.id AND prize_student.student = student_id_t
        ORDER BY id;
    RETURN;
END
$$;