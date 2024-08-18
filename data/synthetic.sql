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
(1010, 'Julia Yellow', 'julia_y', 'token_julia');

-- Insert fake guestbook entries with signatures
INSERT INTO guestbook (message, signature, author_id, author_username)
VALUES
('Hello, world!', 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAACklEQVR4nGMAAQAABQABDQottAAAAABJRU5ErkJggg==', 1, 'alice_j'),
('Great website!', 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAACklEQVR4nGMAAQAABQABDQottAAAAABJRU5ErkJggg==', 2, 'bob_s'),
('Nice to meet you all', 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAACklEQVR4nGMAAQAABQABDQottAAAAABJRU5ErkJggg==', 3, 'charlie_b'),
('Loving the design', 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAACklEQVR4nGMAAQAABQABDQottAAAAABJRU5ErkJggg==', 4, 'diana_p'),
('Keep up the good work!', 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAACklEQVR4nGMAAQAABQABDQottAAAAABJRU5ErkJggg==', 5, 'evan_w'),
('Fantastic job', 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAACklEQVR4nGMAAQAABQABDQottAAAAABJRU5ErkJggg==', 6, 'fiona_g'),
('Impressive site', 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAACklEQVR4nGMAAQAABQABDQottAAAAABJRU5ErkJggg==', 7, 'george_b'),
('Thanks for sharing', 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAACklEQVR4nGMAAQAABQABDQottAAAAABJRU5ErkJggg==', 8, 'hannah_r'),
('Looking forward to more', 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAACklEQVR4nGMAAQAABQABDQottAAAAABJRU5ErkJggg==', 9, 'ian_b'),
('Awesome content', 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAACklEQVR4nGMAAQAABQABDQottAAAAABJRU5ErkJggg==', 10, 'julia_y');

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
