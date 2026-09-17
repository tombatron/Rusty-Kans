import { Application, Controller } from "https://cdn.jsdelivr.net/npm/@hotwired/stimulus@3/dist/stimulus.js";

const app = Application.start();

app.register("card-list", class extends Controller {
    static values = { listId: Number }

    connect() {
        Sortable.create(this.element, {
            group: "cards",
            animation: 150,
            onEnd: (event) => {
                const csrfToken = document.querySelector("turbo-frame#lists").dataset.csrf;
                const cardId = event.item.dataset.cardId;
                const toListId = event.to.dataset.cardListListIdValue;

                if (event.from === event.to) {
                    this.persistSortOrder(this.getCardPositions(Array.from(event.from.querySelectorAll("li"))));
                } else {
                    this.persistSortOrder(
                        this.getCardPositions(Array.from(event.from.querySelectorAll("li"))), 
                        this.getCardPositions(Array.from(event.to.querySelectorAll("li")))
                    );

                    fetch(`/lists/${toListId}/cards/${cardId}/move`, {
                        method: "POST",
                        headers: { 
                            "X-CSRF-TOKEN": csrfToken,
                            "Accept": "text/vnd.turbo-stream.html" 
                        },
                    });                    
                }
            }
        })
    }

    getCardPositions(elements) {
        return elements.map((element, index) => {
            return {
                id: Number(element.dataset.cardId),
                index: index,
            }
        });
    }

    persistSortOrder(...elements) {
        const positions = elements.flat();
        
        if (positions.length > 1) {
            const csrfToken = document.querySelector("turbo-frame#lists").dataset.csrf;

            fetch("/lists/sort_order", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                    "X-CSRF-TOKEN": csrfToken,
                },
                body: JSON.stringify(positions),
            });
        }
    }
});

document.addEventListener("turbo:submit-end", (event) => {
    if (event.detail.success) {
        event.detail.formSubmission.formElement.reset();
    }
});
