import { Icons, List } from "@project-gauntlet/api/components";
import { ReactElement, useRef, useState } from "react";
import { usePromise } from "@project-gauntlet/api/hooks";

export default function ListView(): ReactElement {
    // return usePromiseTestBasic()
    // return usePromiseTestExecuteFalse()
    // return usePromiseTestRevalidate()
    // return usePromiseTestAbortableRevalidate()
    // return usePromiseTestMutate()
    // return usePromiseTestMutateOptimistic()
    // return usePromiseTestMutateOptimisticRollback()
    // return usePromiseTestMutateNoRevalidate()
    // return usePromiseTestThrow()

    const numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
    const [id, setId] = useState("default");

    return (
        <List
            onSelectionChange={id => {
                console.log("onSelectionChange " + id)
                setId(id);
            }}
        >
            {
                numbers.map(value => (
                    <List.Item id={"id" + value} title={"Title " + value}/>
                ))
            }
            <List.Section title={"Selected id: " + id}>
                <List.Section.Item id="id section 1" title="Title Section 1" icon={Icons.Clipboard}/>
            </List.Section>
            <List.Section title="Section 2">
                <List.Section.Item id="id section 2 1" title="Title Section 2 1" subtitle="Subtitle 2 1"/>
                <List.Section.Item id="id section 2 2" title="Title Section 2 2"/>
                <List.Section.Item id="id section 2 3" title="Title Section 2 3" subtitle="Subtitle 2 3"/>
            </List.Section>
        </List>
    )
}

function usePromiseTestBasic(): ReactElement {
    const { data, error, isLoading } = usePromise(
        async (one, two, three) => await inNSec(5),
        [1, 2, 3]
    );

    printState(data, error, isLoading)

    return (
        <List isLoading={isLoading} onSelectionChange={id => {}}>
            <List.Section.Item id="id-1" title="Item ID 1" icon={Icons.Clipboard}/>
        </List>
    )
}

function usePromiseTestExecuteFalse(): ReactElement {
    const { data, error, isLoading } = usePromise(
        async (one, two, three) => await inNSec(5),
        [1, 2, 3],
        {
            execute: false
        }
    );

    printState(data, error, isLoading)

    return (
        <List isLoading={isLoading} onSelectionChange={id => {}}>
            <List.Section.Item id="id-1" title="Item ID 1" icon={Icons.Clipboard}/>
        </List>
    )
}

function usePromiseTestRevalidate(): ReactElement {
    const { data, error, isLoading, revalidate } = usePromise(
        async (one, two, three) => await inNSec(5),
        [1, 2, 3],
    );

    printState(data, error, isLoading)

    return (
        <List isLoading={isLoading} onSelectionChange={id => revalidate()}>
            <List.Section.Item id="id-1" title="Item ID 1" icon={Icons.Clipboard}/>
        </List>
    )
}

function usePromiseTestAbortableRevalidate(): ReactElement {
    const abortable = useRef<AbortController>();

    const { data, error, isLoading, revalidate } = usePromise(
        async (one, two, three) => {
            await inNSec(5)
        },
        [1, 2, 3],
        {
            abortable,
        }
    );

    printState(data, error, isLoading)

    return (
        <List isLoading={isLoading} onSelectionChange={id => revalidate()}>
            <List.Section.Item id="id-1" title="Item ID 1" icon={Icons.Clipboard}/>
        </List>
    )
}

function usePromiseTestMutate(): ReactElement {
    const { data, error, isLoading, mutate } = usePromise(
        async (one, two, three) => await inNSec(5),
        [1, 2, 3],
    );

    printState(data, error, isLoading)

    return (
        <List isLoading={isLoading} onSelectionChange={async id => await mutate(inNSec(5))}>
            <List.Section.Item id="id-1" title="Item ID 1" icon={Icons.Clipboard}/>
        </List>
    )
}

function usePromiseTestMutateOptimistic(): ReactElement {
    const { data, error, isLoading, mutate } = usePromise(
        async (one, two, three) => await inNSec(5),
        [1, 2, 3],
    );

    printState(data, error, isLoading)

    return (
        <List isLoading={isLoading} onSelectionChange={async id => {
            await mutate(
                inNSec(5),
                {
                    optimisticUpdate: data1 => data1 + " optimistic",
                }
            )
        }}>
            <List.Section.Item id="id-1" title={"Item ID 1 " + data} icon={Icons.Clipboard}/>
        </List>
    )
}

function usePromiseTestMutateOptimisticRollback(): ReactElement {
    const { data, error, isLoading, mutate } = usePromise(
        async (one, two, three) => await inNSec(5),
        [1, 2, 3],
    );

    printState(data, error, isLoading)

    return (
        <List isLoading={isLoading} onSelectionChange={async id => {
            const newVar = await mutate(
                new Promise<string>((_resolve, reject) => {
                    setTimeout(
                        () => {
                            reject("fail")
                        },
                        5 * 1000
                    );
                }),
                {
                    optimisticUpdate: data1 => data1 + " optimistic",
                    rollbackOnError:  data1 => data1 + " failed",
                }
            );
        }}>
            <List.Section.Item id="id-1" title={"Item ID 1 " + data} icon={Icons.Clipboard}/>
        </List>
    )
}

function usePromiseTestMutateNoRevalidate(): ReactElement {
    const { data, error, isLoading, mutate } = usePromise(
        async (one, two, three) => await inNSec(5),
        [1, 2, 3],
    );

    printState(data, error, isLoading)

    return (
        <List isLoading={isLoading} onSelectionChange={async id => {
            await mutate(
                inNSec(5),
                {
                    shouldRevalidateAfter: false,
                }
            )
        }}>
            <List.Section.Item id="id-1" title="Item ID 1" icon={Icons.Clipboard}/>
        </List>
    )
}

function usePromiseTestThrow(): ReactElement {
    const { data, error, isLoading } = usePromise(
        async (one, two, three) => {
            throw new Error("test")
        },
        [1, 2, 3],
    );

    printState(data, error, isLoading)

    return (
        <List isLoading={isLoading} onSelectionChange={id => {}}>
            <List.Section.Item id="id-1" title="Item ID 1" icon={Icons.Clipboard}/>
        </List>
    )
}

async function inNSec(n: number): Promise<string> {
    return new Promise<string>(resolve => {
        setTimeout(
            () => {
                resolve(`Promise resolved after ${n} sec: ${Math.random()}`)
            },
            n * 1000
        );
    })
}

function printState(data: any, error: unknown, isLoading: boolean) {
    console.log("")
    console.log("=====")
    console.dir(data)
    console.dir(error)
    console.dir(isLoading)
}