import json
import os

ids = [
    "71f6112a-a67b-413c-868d-cbfc72658006",
    "6cbb9c9c-ae40-4255-93b6-966c95af00fd",
    "9d9e5f5b-321c-4c84-942a-3b5f2b2d0b4b",
    "688917cf-e563-45a8-a4ec-05fe46fe9ff1",
    "d000d988-b031-412b-b08a-700e01cfd3e0",
    "ee3c9d1e-2ab7-4a8a-b1ff-0c52628e16f5",
    "3595b61d-df09-4761-86d9-b878c365e55b",
    "bd200a6f-9266-4e53-944f-ee381cc32a42",
    "d046dbc9-16d8-4c74-b632-d42999342a24",
    "d2da741f-d9f9-476c-85cf-da64f1721a6e"
]

brain_dir = "/Users/sac/.gemini/antigravity-cli/brain"

for cid in ids:
    transcript_path = os.path.join(brain_dir, cid, ".system_generated/logs/transcript_full.jsonl")
    try:
        with open(transcript_path, 'r') as f:
            for line in f:
                if 'PLANNER_RESPONSE' in line:
                    data = json.loads(line)
                    if 'tool_calls' in data:
                        for call in data['tool_calls']:
                            if call['name'] in ['write_to_file', 'replace_file_content', 'multi_replace_file_content']:
                                args = call['args']
                                file_path = args.get('TargetFile', '')
                                if 'chapters/' in file_path:
                                    content = args.get('CodeContent', None)
                                    if content is None:
                                        content = args.get('ReplacementContent', '')
                                    if content:
                                        content = content.replace('autonomous', 'autonomic')
                                        content = content.replace('Autonomous', 'Autonomic')
                                        content = content.replace('Artificial General Intelligence', 'Autonomic General Hyper-Intelligence')
                                        content = content.replace('AGI', 'AGHI')
                                        
                                        with open(file_path, 'w') as out_f:
                                            out_f.write(content)
                                        print(f"Recovered and updated {file_path} from {cid}")
    except Exception as e:
        print(f"Error on {cid}: {e}")
