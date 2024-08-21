-- Insert fake guests
INSERT INTO guests (github_id, name, username, access_token)
VALUES
(1001, 'Alice Johnson', 'alice_j', 'token_alice'),
(1002, 'Bob Smith', 'bob_s', 'token_bob'),
(1003, 'Charlie Brown', 'charlie_b', 'token_charlie'),
(1004, 'Diana Prince', 'diana_p', 'token_diana'),
(1005, 'Evan White', 'evan_w', 'token_evan'),
(1006, 'Fiona Green', 'fiona_g', 'token_fiona'),
(1007, 'George Black', 'george_b', 'token_george'),
(1008, 'Hannah Red', 'hannah_r', 'token_hannah'),
(1009, 'Ian Blue', 'ian_b', 'token_ian'),
(1010, 'Julia Yellow', 'julia_y1', 'token_julia'),
(1101, 'Alice Johnson', 'alice_j1', 'token_alice'),
(1102, 'Bob Smith', 'bob_s1', 'token_bob'),
(1103, 'Charlie Brown', 'charlie_b1', 'token_charlie'),
(1104, 'Diana Prince', 'diana_p1', 'token_diana'),
(1105, 'Evan White', 'evan_w1', 'token_evan'),
(1106, 'Fiona Green', 'fiona_g1', 'token_fiona'),
(1107, 'George Black', 'george_b1', 'token_george'),
(1108, 'Hannah Red', 'hannah_r1', 'token_hannah'),
(1109, 'Ian Blue', 'ian_b1', 'token_ian'),
(1110, 'Julia Yellow', 'julia_y1', 'token_julia');


-- Insert fake guestbook entries with signatures
INSERT INTO guestbook (message, signature, author_id, author_username)
VALUES
('Hello, world!', 'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAACklEQVR4nGMAAQAABQABDQottAAAAABJRU5ErkJggg==', 1, 'alice_j'),
('Great website!', 'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAACklEQVR4nGMAAQAABQABDQottAAAAABJRU5ErkJggg==', 2, 'bob_s'),
('Nice to meet you all', 'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAACklEQVR4nGMAAQAABQABDQottAAAAABJRU5ErkJggg==', 3, 'charlie_b'),
('Loving the design', 'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAACklEQVR4nGMAAQAABQABDQottAAAAABJRU5ErkJggg==', 4, 'diana_p'),
('Keep up the good work!', 'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAACklEQVR4nGMAAQAABQABDQottAAAAABJRU5ErkJggg==', 5, 'evan_w'),
('Fantastic job', 'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAACklEQVR4nGMAAQAABQABDQottAAAAABJRU5ErkJggg==', 6, 'fiona_g'),
('Impressive site', 'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAACklEQVR4nGMAAQAABQABDQottAAAAABJRU5ErkJggg==', 7, 'george_b'),
('Thanks for sharing', 'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAACklEQVR4nGMAAQAABQABDQottAAAAABJRU5ErkJggg==', 8, 'hannah_r'),
('Looking forward to more', 'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAACklEQVR4nGMAAQAABQABDQottAAAAABJRU5ErkJggg==', 9, 'ian_b'),
('Awesome content', 'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAACklEQVR4nGMAAQAABQABDQottAAAAABJRU5ErkJggg==', 10, 'julia_y'),
('Test', 'Test', 11, 'alice_j1'),
('Test', 'Test', 12, 'bob_s1'),
('Test', 'Test', 13, 'charlie_b1'),
('Test', 'Test', 14, 'diana_p1'),
('Test', 'Test', 15, 'evan_w1'),
('Test', 'Test', 16, 'fiona_g1'),
('Test', 'Test', 17, 'george_b1'),
('Test', 'Test', 18, 'hannah_r1'),
('Test', 'Test', 19, 'ian_b1'),
('Test', 'Test', 20, 'julia_y1');


-- Assign guests to the 'guests' group
INSERT INTO guests_groups (guest_id, group_id)
SELECT g.id, gr.id
FROM guests g, groups gr
WHERE gr.name = 'guests';

-- Make Alice an admin
INSERT INTO guests_groups (guest_id, group_id)
SELECT g.id, gr.id
FROM guests g, groups gr
WHERE g.username = 'alice_j' AND gr.name = 'admins';
