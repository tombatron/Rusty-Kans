INSERT INTO boards (name) VALUES ('Whatever'); -- board_id, 1

    INSERT INTO lists (board_id, name) VALUES (1, 'First List'); -- list_id, 1
    
        INSERT INTO cards (list_id, title, description, status) VALUES (1, 'Card 1', 'This is a description', 'Todo'); -- card_id, 1
        INSERT INTO cards (list_id, title, description, status) VALUES (1, 'Card 2', NULL, 'Doing'); -- card_id, 2
        INSERT INTO cards (list_id, title, description, status) VALUES (1, 'Card 3', 'This is another description', 'Done'); -- card_id, 3
        
    INSERT INTO lists (board_id, name) VALUES (1, 'Second List'); -- list_id, 2

        INSERT INTO cards (list_id, title, description, status) VALUES (2, 'Card 4', 'This is a description', 'Todo'); -- card_id, 4
        INSERT INTO cards (list_id, title, description, status) VALUES (2, 'Card 5', NULL, 'Doing'); -- card_id, 5
        INSERT INTO cards (list_id, title, description, status) VALUES (2, 'Card 6', 'This is another description', 'Done'); -- card_id, 6    

INSERT INTO boards (name) VALUES ('Another Board'); -- board_id, 2

    INSERT INTO lists (board_id, name) VALUES (2, 'First List'); -- list_id, 3

        INSERT INTO cards (list_id, title, description, status) VALUES (3, 'Card 6', 'This is a description', 'Todo'); -- card_id, 7
        INSERT INTO cards (list_id, title, description, status) VALUES (3, 'Card 7', NULL, 'Doing'); -- card_id, 8
        INSERT INTO cards (list_id, title, description, status) VALUES (3, 'Card 8', 'This is another description', 'Done'); -- card_id, 9        
    
    INSERT INTO lists (board_id, name) VALUES (2, 'Second List'); -- list_id, 4

        INSERT INTO cards (list_id, title, description, status) VALUES (4, 'Card 9', 'This is a description', 'Todo'); -- card_id, 10
        INSERT INTO cards (list_id, title, description, status) VALUES (4, 'Card 10', NULL, 'Doing'); -- card_id, 11
        INSERT INTO cards (list_id, title, description, status) VALUES (4, 'Card 11', 'This is another description', 'Done'); -- card_id, 12      

INSERT INTO boards (name) VALUES ('A third board?!'); -- board_id, 3

    INSERT INTO lists (board_id, name) VALUES (3, 'First List'); -- list_id, 5

        INSERT INTO cards (list_id, title, description, status) VALUES (5, 'Card 12', 'This is a description', 'Todo'); -- card_id, 13
        INSERT INTO cards (list_id, title, description, status) VALUES (5, 'Card 13', NULL, 'Doing'); -- card_id, 14
        INSERT INTO cards (list_id, title, description, status) VALUES (5, 'Card 14', 'This is another description', 'Done'); -- card_id, 15          
    
    INSERT INTO lists (board_id, name) VALUES (3, 'Second List'); -- list_id, 6

        INSERT INTO cards (list_id, title, description, status) VALUES (6, 'Card 15', 'This is a description', 'Todo'); -- card_id, 16
        INSERT INTO cards (list_id, title, description, status) VALUES (6, 'Card 16', NULL, 'Doing'); -- card_id, 17
        INSERT INTO cards (list_id, title, description, status) VALUES (6, 'Card 17', 'This is another description', 'Done'); -- card_id, 18          
