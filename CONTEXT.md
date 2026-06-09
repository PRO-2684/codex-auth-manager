# Codex Auth Manager

Codex Auth Manager manages named Codex authentication files and switches which one Codex uses.

## Language

**Identity**:
A named saved Codex authentication file. Each **Identity** stores the auth state Codex would normally keep in `auth.json`.
_Avoid_: Profile, account, slot, name

**Identity name**:
The user-facing name of an **Identity**. CAM stores each identity as `<identity name>.json`, so `.json` can be part of the identity name.
_Avoid_: Filename, path, profile name

**Active identity**:
The **Identity** currently selected for Codex to use.
_Avoid_: Current profile, selected account, active slot

**Native auth file**:
The regular Codex `auth.json` file created by Codex itself, before CAM captures it into an **Auth slot**.
_Avoid_: Unmanaged profile, raw auth

**Managed auth file**:
An auth file stored by CAM as an **Identity**.
_Avoid_: Profile file, account file, slot file

**Capture**:
The operation that converts a **Native auth file** into a named **Identity** and makes that identity active.
_Avoid_: Takeover, name

**Use**:
The operation that makes an existing **Identity** become the **Active identity**.
_Avoid_: Switch profile, select account

**Detach**:
The operation that removes Codex's link to the **Active identity** without deleting any **Identity**.
_Avoid_: Login, new

## Example Dialogue

Dev: "I logged in with Codex, so I have a native auth file. How do I save it?"

Domain expert: "Capture the native auth file into a new identity."

Dev: "How do I switch back to a saved login?"

Domain expert: "Use the identity you want; it becomes the active identity."

Dev: "How do I let Codex create a fresh native auth file?"

Domain expert: "Detach from the active identity first, then let Codex create a native auth file."
