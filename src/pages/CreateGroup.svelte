<script lang="ts">
    import { Page, Navbar, Link, BlockTitle, List, ListItem } from "framework7-svelte";

    // Todo: Create group page
    // This allows you to create a new group with more than 1 other person
    // 1. Search/select contacts to add to the group
    // - should only show contacts that aren't already in the group
    // - should only show contacts that have a prekey published

    // 2. Set group name
    // 3. Set group description
    // 4. Set group image
    // 5. Create group
    // - create MLS group
    // - invite people to the group
    // - send welcome messages

    // 6. Redirect to the group chat page
</script>

<Page class="create-group-page bg-gray-900">
    <Navbar backLink title="Create Group">
        <Link slot="right" popupClose>Cancel</Link>
    </Navbar>
    <BlockTitle>Create Group</BlockTitle>
    <List>
        <ListItem>1. Search for contacts to add to the group</ListItem>
        <ListItem>2. Set group name</ListItem>
        <ListItem>3. Set group description</ListItem>
        <ListItem>4. Set group image</ListItem>
        <ListItem>
            5. Create MLS group
            <List>
                <ListItem>Create credentials and group with each members prekeys</ListItem>
                <ListItem>Send first commit</ListItem>
                <ListItem>Send welcome messages</ListItem>
            </List>
        </ListItem>
        <ListItem>6. Redirect to the group chat page</ListItem>
    </List>
</Page>
