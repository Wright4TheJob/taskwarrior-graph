# Taskwarrior-graph
Interactive dependancy graph and work breakdown structure for taskwarrior.

The program reads the task list from TaskWarrior with the filters selected, arranges them elegantly using Graphviz as a backend, and allows dynamic constraints to be added visually by dragging and dropping. The graph will update to relfect the new dependancies.

Feature list status:
- [ ] Read from TaskWarrior
- [ ] Filter according to project
- [ ] Filter according to tag
- [ ] Filter only active
- [ ] Send task data to graphviz
- [ ] Parse and render results from GraphViz
- [x] Create new link by dragging and dropping
- [ ] Select and delete link with a click and "delete" key
- [ ] View pending TaskWarrior commands
- [ ] Save actions to TaskWarrior
