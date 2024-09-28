from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import StreamingResponse, JSONResponse
from pydantic import BaseModel
from typing import Optional
from anthropic import AsyncAnthropic
from uvicorn import Config, Server
import json
import logging
import asyncio
import signal

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

class CVText(BaseModel):
    text: str
    anthropic_api_key: str

class JobApplication(BaseModel):
    anthropic_api_key: str
    job_title: Optional[str] = None
    company_name: Optional[str] = None
    job_description: Optional[str] = None
    key_requirements: Optional[str] = None
    applicant_name: Optional[str] = None
    applicant_experience: Optional[str] = None
    applicant_skills: Optional[str] = None
    applicant_projects: Optional[str] = None
    applicant_education: Optional[str] = None
    applicant_certificates: Optional[str] = None

    class Config:
        from_attributes = True


app = FastAPI()

# Enable CORS
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


async def shutdown(signal, loop):
    print(f"Received exit signal {signal.name}...")
    tasks = [t for t in asyncio.all_tasks() if t is not asyncio.current_task()]
    [task.cancel() for task in tasks]
    print(f"Cancelling {len(tasks)} outstanding tasks")
    await asyncio.gather(*tasks, return_exceptions=True)
    loop.stop()


def handle_exception(loop, context):
    msg = context.get("exception", context["message"])
    print(f"Caught exception: {msg}")
    print("Shutting down...")
    asyncio.create_task(shutdown(signal.SIGTERM, loop))


async def main():
    loop = asyncio.get_running_loop()
    signals = (signal.SIGHUP, signal.SIGTERM, signal.SIGINT)
    for s in signals:
        loop.add_signal_handler(
            s, lambda s=s: asyncio.create_task(shutdown(s, loop)))

    loop.set_exception_handler(handle_exception)

    config = Config(app=app, host="0.0.0.0", port=8080)
    server = Server(config)

    await server.serve()


@app.post("/api/suggestions")
async def generate_cover_letter(job_data: JobApplication):
    try:
        client = AsyncAnthropic(api_key=job_data.anthropic_api_key)
        response = await client.messages.create(
            model="claude-3-opus-20240229",
            max_tokens=1000,
            messages=[
                {
                    "role": "user",
                    "content": f"""Job Title: {job_data.job_title}
Company: {job_data.company_name}
Job Description: {job_data.job_description}
Key Requirements: {job_data.key_requirements}

Applicant Name: {job_data.applicant_name}
Applicant Experience: {job_data.applicant_experience}
Applicant Certified Skills: {job_data.applicant_certificates}
Applicant Projects: {job_data.applicant_projects}
Applicant Education: {job_data.applicant_education}

Based on the above information, generate a tailored cover letter for {job_data.applicant_name} applying for the {job_data.job_title} position at {job_data.company_name}. The cover letter should:

1. Address the specific requirements mentioned in the job description
2. Highlight the applicant's relevant experiences and skills
3. Demonstrate enthusiasm for the role and company
4. Be professional and well-structured over no more than 4 paragraphs
5. Be approximately 500 words in length
6. Be written in Italian
7. Use the following format, ensuring there's a line break between paragraphs:
   <p>Paragraph 1</p>

   <p>Paragraph 2</p>

   <p>Paragraph 3</p>

   <p>Paragraph 4</p>

Generate only the cover letter:""",
                }
            ],
            temperature=0.7,
            top_p=0.9,
            system="You are an AI assistant specialized in creating tailored cover letters based on job descriptions and applicant details. Your task is to generate a professional and compelling cover letter that matches the provided job description and highlights the applicant's relevant skills and experiences. Write the cover letter in Italian.",
        )

        # Extract the text content from the response
        cover_letter_text = response.content[0].text if response.content else ""

        # Return a dictionary with the cover_letter key
        return {"cover_letter": cover_letter_text}

    except Exception as e:
        print(f"Error in generating cover letter: {str(e)}")
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/api/cv")
async def analyze_cv(cv: CVText):
    try:
        client = AsyncAnthropic(api_key=cv.anthropic_api_key)
        message = await client.messages.create(
            model="claude-3-opus-20240229",
            max_tokens=1500,
            messages=[
                {
                    "role": "user",
                    "content": f"""Analyze the following CV text and extract relevant information for the following categories:
1. Experience
2. Interests
3. Projects
4. Education
5. Certificates

CV Text:
{cv.text}

Please provide a JSON response with these categories as keys, and lists of relevant sentences or phrases as values. If a category is not found in the CV, return an empty list for that category.
""",
                }
            ],
            temperature=0.2,
            top_p=0.9,
            system="You are an AI assistant specialized in analyzing CVs and extracting relevant information. Your task is to process the given CV text and categorize information into experience, interests, projects, education, and certificates. Provide your analysis in a structured JSON format.",
        )

        # Handle the case where content might be a list
        if isinstance(message.content, list):
            response_text = message.content[0].text
        else:
            response_text = message.content

        logger.info(f"Response text: {response_text}")

        # Check if response_text is a string before using strip()
        if isinstance(response_text, str):
            response_text = response_text.strip()
        else:
            raise ValueError("Unexpected response format from Claude API")

        # Find the start and end of the JSON object
        json_start = response_text.find("{")
        json_end = response_text.rfind("}") + 1

        if json_start == -1 or json_end == -1:
            raise ValueError("JSON object not found in the response")

        json_str = response_text[json_start:json_end]

        # Parse the JSON response from Claude
        analysis_result = json.loads(json_str)

        logger.info(f"Analysis results: {analysis_result}")
        return analysis_result
    except json.JSONDecodeError as json_error:
        logger.error(f"JSON parsing error: {str(json_error)}")
        raise HTTPException(
            status_code=500,
            detail=f"Invalid JSON response from Claude: {str(json_error)}",
        )
    except Exception as e:
        logger.error(f"Error in analyze_cv: {str(e)}")
        raise HTTPException(status_code=500, detail=str(e))


if __name__ == "__main__":
    asyncio.run(main())
