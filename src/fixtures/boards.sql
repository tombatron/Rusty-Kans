INSERT INTO boards (name) VALUES ('Whatever'); -- board_id, 1

    INSERT INTO lists (board_id, name) VALUES (1, 'First List'); -- list_id, 1
    
        INSERT INTO cards (list_id, title, description) VALUES (1, 'Card 1', 'This is a description'); -- card_id, 1
        INSERT INTO cards (list_id, title, description) VALUES (1, 'Card 2', NULL); -- card_id, 2
        INSERT INTO cards (list_id, title, description) VALUES (1, 'Card 3', 'This is another description'); -- card_id, 3
        
    INSERT INTO lists (board_id, name) VALUES (1, 'Second List'); -- list_id, 2

        INSERT INTO cards (list_id, title, description) VALUES (2, 'Card 4', 'This is a description'); -- card_id, 4
        INSERT INTO cards (list_id, title, description) VALUES (2, 'Card 5', NULL); -- card_id, 5
        INSERT INTO cards (list_id, title, description) VALUES (2, 'Card 6', 'This is another description'); -- card_id, 6

INSERT INTO boards (name) VALUES ('Another Board'); -- board_id, 2

    INSERT INTO lists (board_id, name) VALUES (2, 'First List'); -- list_id, 3

        INSERT INTO cards (list_id, title, description) VALUES (3, 'Card 6', 'This is a description'); -- card_id, 7
        INSERT INTO cards (list_id, title, description) VALUES (3, 'Card 7', NULL); -- card_id, 8
        INSERT INTO cards (list_id, title, description) VALUES (3, 'Card 8', 'This is another description'); -- card_id, 9
    
    INSERT INTO lists (board_id, name) VALUES (2, 'Second List'); -- list_id, 4

        INSERT INTO cards (list_id, title, description) VALUES (4, 'Card 9', 'This is a description'); -- card_id, 10
        INSERT INTO cards (list_id, title, description) VALUES (4, 'Card 10', NULL); -- card_id, 11
        INSERT INTO cards (list_id, title, description) VALUES (4, 'Card 11', 'This is another description'); -- card_id, 12

INSERT INTO boards (name) VALUES ('A third board?!'); -- board_id, 3

    INSERT INTO lists (board_id, name) VALUES (3, 'First List'); -- list_id, 5

        INSERT INTO cards (list_id, title, description) VALUES (5, 'Card 12', 'This is a description'); -- card_id, 13
        INSERT INTO cards (list_id, title, description) VALUES (5, 'Card 13', NULL); -- card_id, 14
        INSERT INTO cards (list_id, title, description) VALUES (5, 'Card 14', 'This is another description'); -- card_id, 15
    
    INSERT INTO lists (board_id, name) VALUES (3, 'Second List'); -- list_id, 6

        INSERT INTO cards (list_id, title, description) VALUES (6, 'Card 15', 'This is a description'); -- card_id, 16
        INSERT INTO cards (list_id, title, description) VALUES (6, 'Card 16', NULL); -- card_id, 17
        INSERT INTO cards (list_id, title, description) VALUES (6, 'Card 17', 'This is another description'); -- card_id, 18

INSERT INTO users (user_id, source, oauth_login, display_name, avatar_url) VALUES (-10000, 'dev', 'dev_login', 'test user', 'http://example.com/whatever.gif');
INSERT INTO users (user_id, source, oauth_login, display_name, avatar_url) VALUES (-20000, 'dev', 'dev_login_2', 'test user 2', 'http://example.com/whatever_2.gif');
INSERT INTO users (user_id, source, oauth_login, display_name, avatar_url) VALUES (-30000, 'dev', 'dev_login_3', 'test user 3', 'http://example.com/whatever_3.gif');

INSERT INTO board_access (board_id, granted_to_user_id, granted_to_user_source, permission, granted_at) VALUES (1, -20000, 'dev', 'edit', '2026-09-26T14:32:07+00:00');
INSERT INTO board_access (board_id, granted_to_user_id, granted_to_user_source, permission, granted_at) VALUES (1, -30000, 'dev', 'edit', '2026-09-26T15:32:07+00:00');

INSERT INTO board_access_grants (owner_database_id, remote_board_id, remote_board_name, created_at) VALUES ('whatever', 1, 'board 1', '2026-09-26T15:32:07+00:00');
INSERT INTO board_access_grants (owner_database_id, remote_board_id, remote_board_name, created_at) VALUES ('whatever 2', 2, 'board 2', '2026-09-26T15:32:07+00:00');