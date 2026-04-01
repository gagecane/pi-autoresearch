# You are running in a task identification -> review -> feedback loop to build pi-autoresearch. Follow these instructions exactly

- You must only work on a single task
- Read this repo and understand what this product does
- Read memories.md
- Read tasks.md and choose the next highest priority task to do
- If there are no more tasks to do, your task is to research the code and discover tasks that need done. This can include refactoring, improved line and branch coverage, mutatation tests, contract tests, and new features, improve documentation in docs/, align specs in the specs/ folder with the code implementation, improve test suite runtime, improve the README.md, etc. Add the tasks to tasks.md.
- REVIEW tasks take the highest priority. When doing a REVIEW task, carefully review the changes made for correctness. Do not make any code changes. If you have feedback, write it to feedback.md and mark the task as REVISE. If the task is completed without feedback, mark it as COMPLETE
- REVISE tasks take the second highest priority. Read feedback.md and implement the task, then mark it ready for REVIEW again and erase the feedback from feedback.md
- If the task you selected is too abstract, your task is to decompose it to smaller tasks
- When you are done with your task:
  - record important learnings in memories.md
  - Update your progress in progress.md with a timestamp.
- If you finished a task, mark it as ready for REVIEW. Tasks must be reviewed and can only be marked COMPLETE by the reviewer.
- Commit your changes. Do not push them. You are done for now.
- If there are absolutely no more tasks to do write <promise>COMPLETE</promise> to progress.md
