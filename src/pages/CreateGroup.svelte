<script lang="ts">
    import { Page, Navbar, Link, BlockTitle, List, ListItem } from "framework7-svelte";

    // TODO: Create group page
    // This allows you to create a new group with more than 1 other person
    // 1. Search/select contacts to add to the group
    // - should only show contacts that aren't already in the group
    // - should only show contacts that have a prekey published

    // 2. Set group name
    // 3. Set group description
    // 4. Group image is just letters of name + derived color
    // 5. Create group
    // 6. Redirect to the group chat page
</script>

<Page class="create-group-page bg-gray-900">
    <Navbar backLink title="Create Group">
        <Link slot="right" popupClose>Cancel</Link>
    </Navbar>
    <BlockTitle>Create Group</BlockTitle>
    <List>
        <ListItem>Didn't I already say? Two Weeks™.</ListItem>
    </List>
</Page>
