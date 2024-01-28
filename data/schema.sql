--
-- PostgreSQL database dump
--

-- Dumped from database version 15.4
-- Dumped by pg_dump version 15.4

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: images; Type: TABLE; Schema: public; Owner: rileybell
--

CREATE TABLE public.images (
    id integer NOT NULL,
    user_id integer NOT NULL,
    image bytea NOT NULL,
    reference_count integer DEFAULT 1 NOT NULL,
    mime_type character varying(255) NOT NULL
);


ALTER TABLE public.images OWNER TO rileybell;

--
-- Name: images_id_seq; Type: SEQUENCE; Schema: public; Owner: rileybell
--

CREATE SEQUENCE public.images_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.images_id_seq OWNER TO rileybell;

--
-- Name: images_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: rileybell
--

ALTER SEQUENCE public.images_id_seq OWNED BY public.images.id;


--
-- Name: notes; Type: TABLE; Schema: public; Owner: rileybell
--

CREATE TABLE public.notes (
    id integer NOT NULL,
    user_id integer NOT NULL,
    content text NOT NULL,
    update_time bigint NOT NULL,
    title text DEFAULT ''::character varying NOT NULL,
    favourite boolean DEFAULT false NOT NULL,
    is_diary boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


ALTER TABLE public.notes OWNER TO rileybell;

--
-- Name: notes_id_seq; Type: SEQUENCE; Schema: public; Owner: rileybell
--

CREATE SEQUENCE public.notes_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.notes_id_seq OWNER TO rileybell;

--
-- Name: notes_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: rileybell
--

ALTER SEQUENCE public.notes_id_seq OWNED BY public.notes.id;


--
-- Name: sessions; Type: TABLE; Schema: public; Owner: rileybell
--

CREATE TABLE public.sessions (
    id character varying(256) NOT NULL,
    user_id integer NOT NULL
);


ALTER TABLE public.sessions OWNER TO rileybell;

--
-- Name: users; Type: TABLE; Schema: public; Owner: rileybell
--

CREATE TABLE public.users (
    id integer NOT NULL,
    email character varying(255) NOT NULL,
    password character varying(255) NOT NULL
);


ALTER TABLE public.users OWNER TO rileybell;

--
-- Name: users_id_seq; Type: SEQUENCE; Schema: public; Owner: rileybell
--

CREATE SEQUENCE public.users_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.users_id_seq OWNER TO rileybell;

--
-- Name: users_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: rileybell
--

ALTER SEQUENCE public.users_id_seq OWNED BY public.users.id;


--
-- Name: images id; Type: DEFAULT; Schema: public; Owner: rileybell
--

ALTER TABLE ONLY public.images ALTER COLUMN id SET DEFAULT nextval('public.images_id_seq'::regclass);


--
-- Name: notes id; Type: DEFAULT; Schema: public; Owner: rileybell
--

ALTER TABLE ONLY public.notes ALTER COLUMN id SET DEFAULT nextval('public.notes_id_seq'::regclass);


--
-- Name: users id; Type: DEFAULT; Schema: public; Owner: rileybell
--

ALTER TABLE ONLY public.users ALTER COLUMN id SET DEFAULT nextval('public.users_id_seq'::regclass);


--
-- Name: images images_pkey; Type: CONSTRAINT; Schema: public; Owner: rileybell
--

ALTER TABLE ONLY public.images
    ADD CONSTRAINT images_pkey PRIMARY KEY (id);


--
-- Name: notes notes_pkey; Type: CONSTRAINT; Schema: public; Owner: rileybell
--

ALTER TABLE ONLY public.notes
    ADD CONSTRAINT notes_pkey PRIMARY KEY (id);


--
-- Name: sessions sessions_pkey; Type: CONSTRAINT; Schema: public; Owner: rileybell
--

ALTER TABLE ONLY public.sessions
    ADD CONSTRAINT sessions_pkey PRIMARY KEY (id);


--
-- Name: users users_pkey; Type: CONSTRAINT; Schema: public; Owner: rileybell
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (id);


--
-- Name: images images_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: rileybell
--

ALTER TABLE ONLY public.images
    ADD CONSTRAINT images_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- Name: notes notes_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: rileybell
--

ALTER TABLE ONLY public.notes
    ADD CONSTRAINT notes_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- Name: sessions sessions_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: rileybell
--

ALTER TABLE ONLY public.sessions
    ADD CONSTRAINT sessions_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- PostgreSQL database dump complete
--

