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