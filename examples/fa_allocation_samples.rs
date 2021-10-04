//! Example FA allocation XML request data

///Replace all with your own accountIds

//==================================================================================================
pub const FA_ONE_GROUP: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?> \
                                <ListOfGroups> \
                                <Group> \
                                <name>Equal_Quantity</name> \
                                <ListOfAccts varName=\"list\"> \
                                <String>DU119915</String> \
                                <String>DU119916</String> \
                                </ListOfAccts> \
                                <defaultMethod>EqualQuantity</defaultMethod> \
                                </Group> \
                                </ListOfGroups>";

//==================================================================================================
pub const FA_TWO_GROUPS: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?> \
                            <ListOfGroups> \
                            <Group> \
                            <name>Equal_Quantity</name> \
                            <ListOfAccts varName=\"list\"> \
                            <String>DU119915</String> \
                            <String>DU119916</String> \
                            </ListOfAccts> \
                            <defaultMethod>EqualQuantity</defaultMethod> \
                            </Group> \
                            <Group> \
                            <name>Pct_Change</name> \
                            <ListOfAccts varName=\"list\">
                            <String>DU119915</String> \
                            <String>DU119916</String> \
                            </ListOfAccts> \
                            <defaultMethod>PctChange</defaultMethod> \
                            </Group> \
                            </ListOfGroups>";

//==================================================================================================
pub const FA_ONE_PROFILE: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?> \
                                <ListOfAllocationProfiles> \
                                <AllocationProfile> \
                                <name>Percent_60_40</name> \
                                <type>1</type> \
                                <ListOfAllocations varName=\"listOfAllocations\"> \
                                <Allocation>
                                <acct>DU119915</acct> \
                                <amount>60.0</amount> \
                                </Allocation> \
                                <Allocation> \
                                <acct>DU119916</acct> \
                                <amount>40.0</amount> \
                                </Allocation> \
                                </ListOfAllocations> \
                                </AllocationProfile> \
                                </ListOfAllocationProfiles>";

//==================================================================================================
pub const FA_TWO_PROFILES: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?> \
                                <ListOfAllocationProfiles> \
                                <AllocationProfile> \
                                <name>Percent_60_40</name> \
                                <type>1</type> \
                                <ListOfAllocations varName=\"listOfAllocations\"> \
                                <Allocation> \
                                <acct>DU119915</acct> \
                                <amount>60.0</amount> \
                                </Allocation> \
                                <Allocation> \
                                <acct>DU119916</acct> \
                                <amount>40.0</amount> \
                                </Allocation> \
                                </ListOfAllocations> \
                                </AllocationProfile> \
                                <AllocationProfile> \
                                <name>Ratios_2_1</name> \
                                <type>1</type> \
                                <ListOfAllocations varName=\"listOfAllocations\"> \
                                <Allocation> \
                                <acct>DU119915</acct> \
                                <amount>2.0</amount> \
                                </Allocation> \
                                <Allocation>
                                <acct>DU119916</acct> \
                                <amount>1.0</amount> \
                                </Allocation> \
                                </ListOfAllocations> \
                                </AllocationProfile> \
                                </ListOfAllocationProfiles>";
