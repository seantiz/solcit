import os
import re
import sys
import time
import json
import random
import urllib
import sqlite3
import logging
import requests
from dotenv import load_dotenv
from pathlib import Path
from fuzzywuzzy import fuzz
from datetime import datetime
from bs4 import BeautifulSoup
from requests.adapters import HTTPAdapter
from requests.packages.urllib3.util.retry import Retry

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

class WidenSearchString:
    def __init__(self):
        self.synonyms = {
            "software engineer": ["developer", "programmer", "programmatore", "sviluppatore"],
            "teacher": ["docente", "insegnante", "tutor"],
            "inglese": ["english", "madrelingua inglese"],
            "madrelingua": ["english", "mothertongue", "native speaker"],
            "insegnante": ["teacher", "docente", "tutor"],
        }

    def add_to_original_search_string(self, jobsearch):
        wider_search_string = [jobsearch]
        jobtitles = jobsearch.lower().split()
        for word in jobtitles:
            if word in self.synonyms:
                for synonym in self.synonyms[word]:
                    added_search_string = ' '.join([synonym if w == word else w for w in jobtitles])
                    wider_search_string.append(added_search_string)
        return wider_search_string

class JobSearchEngine:
    def __init__(self):
        self.widen_search = WidenSearchString()
        self.fetch_job_listings = Jooble()

    def search(self, jobsearch, location):
        # The wider search string is for parsing the actual HTML/JSON content on listings pages
        # But our filter has the final say on job listings returned to frontend
        wide_jobsearch = self.widen_search.add_to_original_search_string(jobsearch)
        
        wide_joblistings = []
        for js in wide_jobsearch:
            indeed_results = self.fetch_job_listings.fetch_jobs(js, location)
            wide_joblistings.extend(indeed_results)
        
        filtered_listings = self.filter_listings(wide_joblistings, wide_jobsearch, jobsearch)
        return filtered_listings

    def filter_listings(self, joblistings, wide_jobsearch, original_job_search):
        filtered_listings = []
        for job in joblistings:
            job_title = job.get('title', '').lower()
            
            expanded_match = any(fuzz.partial_ratio(eq.lower(), job_title) > 80 for eq in wide_jobsearch)
            original_match = fuzz.partial_ratio(original_job_search.lower(), job_title) > 70
            
            if expanded_match or original_match:
                filtered_listings.append(job)
        
        return filtered_listings

    def push_listings_to_db(self, conn, jobs):
        c = conn.cursor()
        fetched_date = datetime.now().isoformat()
        new_jobs = 0
        
        for job in jobs:
            title = job.get('title', 'N/A')
            company = job.get('company', 'N/A')
            uniqueid = make_uid(title, company)
            location = job.get('location', 'N/A')
            salary = job.get('salary', 'N/A')
            jobkey = job.get('jobkey', 'N/A')
            
            c.execute("SELECT uniqueid FROM jobs WHERE uniqueid = ?", (uniqueid,))
            existing_job = c.fetchone()
            
            if existing_job is None:
                c.execute('''INSERT INTO jobs 
                             (uniqueid, title, company, location, salary, jobkey, fetched_date, read, appliedto, source)
                             VALUES (?,?,?,?,?,?,?,?,?,?)''',
                          (uniqueid, title, company, location, salary, jobkey, fetched_date, 0, 0, 'jooble'))
                new_jobs += 1
            else:
                c.execute('''UPDATE jobs 
                             SET title=?, company=?, location=?, salary=?, jobkey=?, fetched_date=?
                             WHERE uniqueid=?''',
                          (title, company, location, salary, jobkey, fetched_date, uniqueid))
        
        conn.commit()
        return new_jobs

    @staticmethod
def create_database(db_path):
    conn = sqlite3.connect(db_path)
    c = conn.cursor()
    c.execute('''CREATE TABLE IF NOT EXISTS jobs
                (id INTEGER PRIMARY KEY AUTOINCREMENT,
                uniqueid TEXT,
                title TEXT,
                company TEXT,
                location TEXT,
                salary TEXT,
                jobkey TEXT,
                fetched_date TEXT,
                read INTEGER DEFAULT 0,
                appliedto INTEGER DEFAULT 0,
                source TEXT,
                unique(jobkey,source),
                unique(uniqueid))''')
    c.execute('''CREATE TABLE IF NOT EXISTS stats
                (id INTEGER PRIMARY KEY AUTOINCREMENT,
                uniquejobs INTEGER)''')
    conn.commit()
    return conn

class Jooble:
    def __init__(self):
        self.user_agents = [
            ('Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36', 'chrome'),
            ('Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.0 Safari/605.1.15', 'safari'),
            ('Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:89.0) Gecko/20100101 Firefox/89.0', 'firefox')
        ]

    def bot_facade(self, url, keywords):
        user_agent, browser = random.choice(self.user_agents)
        
        encoded_keywords = urllib.parse.quote_plus(keywords)
        referer = {
            'chrome': f'https://www.google.com/search?q={encoded_keywords}',
            'safari': f'https://www.google.com/search?client=safari&rls=en&q={encoded_keywords}&ie=UTF-8&oe=UTF-8',
            'firefox': f'https://www.google.com/search?client=firefox-b-d&q={encoded_keywords}'
        }

        headers = {
            'User-Agent': user_agent,
            'Accept-Language': 'en-US,en;q=0.9',
            'Accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,image/apng,*/*;q=0.8',
            'DNT': '1',
            'Referer': referer[browser],
            'Connection': 'keep-alive',
            'Upgrade-Insecure-Requests': '1'
        }

        session = requests.Session()
        retry = Retry(total=3, backoff_factor=0.1)
        adapter = HTTPAdapter(max_retries=retry)
        session.mount('http://', adapter)
        session.mount('https://', adapter)
        session.headers.update(headers)

        try:
            time.sleep(random.uniform(1, 3))
            response = session.get(url)
            response.raise_for_status()
            decompressed_content = response.content
            try:
                return decompressed_content.decode('utf-8')
            except UnicodeDecodeError:
                return decompressed_content.decode('iso-8859-1')
        except requests.exceptions.RequestException as e:
            logger.error(f"An error occurred: {e}")
            return None
            
    def set_jooble_url(self, keywords, location, max_pages=10):
        encoded_keywords = urllib.parse.quote_plus(keywords)
        base_url = f"https://it.jooble.org/SearchResult?ukw={encoded_keywords}"
        if location:
            encoded_location = urllib.parse.quote_plus(location)
            base_url += f"&rgns={encoded_location}"
        return self.push_jooble_pages(base_url, keywords, max_pages)

    def push_jooble_pages(self, base_url, keywords, max_pages):
        all_jobs = []
        for page in range(max_pages):
            start = page * 10
            url = f"{base_url}&start={start}"
            logger.info(f"Fetching Jooble page {page + 1}...")
            html_content = self.bot_facade(url, keywords)
            if not html_content:
                logger.error(f"Failed to fetch content for Jooble page {page + 1}")
                break

            job_listings = self.parse_jooble_pages(html_content)
            if not job_listings:
                logger.warning(f"No job listings found on Jooble page {page + 1}")
                break

            all_jobs.extend(job_listings)
            time.sleep(random.uniform(2, 5))

            if len(job_listings) < 10:
                break

        return all_jobs

    def parse_jooble_pages(self, html_content):
        soup = BeautifulSoup(html_content, 'html.parser')
        job_listings = []
    
        # Find all job card divs
        job_cards = soup.find_all('div', {'data-test-name': '_jobCard'})
        
        for card in job_cards:
            try:
                # Extract job title
                title_elem = card.find('h2', class_='sXM9Eq')
                title = title_elem.text.strip() if title_elem else 'N/A'
                
                # Extract company name
                company_elem = card.find('p', class_='z6WlhX')
                company = company_elem.text.strip() if company_elem else 'N/A'
                
                # Extract location
                location_elem = card.find('div', class_='caption NTRJBV')
                location = location_elem.text.strip() if location_elem else 'N/A'
                
                # Extract posted time
                time_elem = card.find('div', class_='caption Vk-5Da')
                posted_time = time_elem.text.strip() if time_elem else 'N/A'
                
                # Extract job description snippet
                desc_elem = card.find('div', class_='PAM72f')
                description = desc_elem.text.strip() if desc_elem else 'N/A'
                
                # Extract job ID
                job_id = card.get('id', 'N/A')
                
                job_listings.append({
                    'title': title,
                    'company': company,
                    'location': location,
                    'posted_time': posted_time,
                    'description': description,
                    'jobkey': job_id
                })
                
            except Exception as e:
                logger.error(f"Error parsing job card: {str(e)}")
        
        return job_listings

    def fetch_jobs(self, keywords, location, max_pages=10):
        # Public method for JobSearchEngines()'s call
        return self.set_jooble_url(keywords, location, max_pages)

def main(db_path, keywords, location):
    search_engine = JobSearchEngine()
    conn = JobSearchEngine.create_database(db_path)
    results = search_engine.search(keywords, location)
    
    conn = sqlite3.connect(db_path)
    new_jooble_jobs = search_engine.push_listings_to_db(conn, results)
    
    logger.info(f"Added {new_jooble_jobs} new Jooble jobs to the database.")
    logger.info(f"Total filtered jobs: {len(results)}")
    
    conn.close()

if __name__ == "__main__":
    if len(sys.argv) < 4:
        print("Usage: python search_engine.py <db_path> <keywords> <location>")
        sys.exit(1)
    
    db_path = sys.argv[1]
    keywords = sys.argv[2]
    location = sys.argv[3]
    
    main(db_path, keywords, location)
