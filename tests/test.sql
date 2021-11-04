DELETE from public.documents;

DELETE FROM public.users;
INSERT INTO public.users(
	id, email, name, created, last_login)
	VALUES 
		('b9518d55-3256-4b96-81d0-65b1d7c4fb31', 'test1@dataregi.com', 'Test User 1', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
		('b9518d55-3256-4b96-81d0-65b1d7c4fb32', 'test2@dataregi.com', 'Test User 2', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
		('b9518d55-3256-4b96-81d0-65b1d7c4fb33', 'test3@dataregi.com', 'Test User 3', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
		('b9518d55-3256-4b96-81d0-65b1d7c4fb34', 'test4@dataregi.com', 'Test User 4', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
	;

