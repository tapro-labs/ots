# Feature Toggles

**Current Features:**
- [Slack](./how-to-integrate-with-slack.md)

### How to enable

Both `backend` and `frontend` have an environment that is used to toggle features on/off. 

- Backend env: `ENABLED_FEATURES` 
- Frontend env: `VITE_ENABLED_FEATURES`

To list which features we want to enable we separate each feature with commas.

Example: `ENABLED_FEATURES="slack,test,test1"`
